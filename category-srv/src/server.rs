use std::sync::Arc;

use blog_proto::{
    category_exists_request::Condition, category_service_server::CategoryService,
    CategoryExistsReply, CategoryExistsRequest, CreateCategoryReply, CreateCategoryRequest,
    EditCategoryReply, EditCategoryRequest, GetCategoryReply, GetCategoryRequest,
    ListCategoryReply, ListCategoryRequest, ToggleCategoryReply, ToggleCategoryRequest,
};
use sqlx::{PgPool, Row};
use tonic::Response;

pub struct Category {
    pool: Arc<PgPool>,
}

impl Category {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

#[tonic::async_trait]
impl CategoryService for Category {
    async fn category_exists(
        &self,
        request: tonic::Request<CategoryExistsRequest>,
    ) -> Result<tonic::Response<CategoryExistsReply>, tonic::Status> {
        let request = request.into_inner();
        let condition = request
            .condition
            .ok_or(tonic::Status::invalid_argument("参数错误"))?;
        let query = match condition {
            Condition::Name(name) => {
                sqlx::query("SELECT COUNT(*) FROM categories WHERE name=$1").bind(name)
            }
            Condition::Id(id) => {
                sqlx::query("SELECT COUNT(*) FROM categories WHERE id=$1").bind(id)
            }
        };
        let row = query
            .fetch_one(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        let count: i64 = row.get(0);
        let reply = CategoryExistsReply { exists: count > 0 };
        Ok(tonic::Response::new(reply))
    }

    async fn create_category(
        &self,
        request: tonic::Request<CreateCategoryRequest>,
    ) -> Result<tonic::Response<CreateCategoryReply>, tonic::Status> {
        let CreateCategoryRequest { name } = request.into_inner();
        let exists_request = tonic::Request::new(CategoryExistsRequest {
            condition: Some(Condition::Name(name.clone())),
        });
        let exists_reply = self.category_exists(exists_request).await?.into_inner();
        if exists_reply.exists {
            return Err(tonic::Status::already_exists("分类已存在"));
        }
        let res = sqlx::query("insert into categories (name) values ($1) returning id")
            .bind(name)
            .fetch_one(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        let reply = CreateCategoryReply { id: res.get("id") };
        Ok(tonic::Response::new(reply))
    }

    async fn edit_category(
        &self,
        request: tonic::Request<EditCategoryRequest>,
    ) -> Result<tonic::Response<EditCategoryReply>, tonic::Status> {
        let EditCategoryRequest { id, name } = request.into_inner();
        let row = sqlx::query("select count(*) from categories where name=$1 and id<>$2")
            .bind(&name)
            .bind(id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        let count: i64 = row.get(0);
        if count > 0i64 {
            return Err(tonic::Status::already_exists("分类已存在"));
        }
        let rows_affected = sqlx::query("update categories set name=$1 where id=$2")
            .bind(&name)
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?
            .rows_affected();
        let reply = EditCategoryReply {
            id,
            ok: rows_affected > 0,
        };
        Ok(tonic::Response::new(reply))
    }

    async fn get_category(
        &self,
        request: tonic::Request<GetCategoryRequest>,
    ) -> Result<tonic::Response<GetCategoryReply>, tonic::Status> {
        let GetCategoryRequest { id, is_del } = request.into_inner();
        let query = match is_del {
            Some(is_del) => {
                sqlx::query("select id,name,is_del from categories where id=$1 and is_del=$2")
                    .bind(id)
                    .bind(is_del)
            }
            None => sqlx::query("select id,name,is_del from categories where id=$1").bind(is_del),
        };
        let row = query
            .fetch_optional(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        let reply = match row {
            Some(row) => GetCategoryReply {
                category: Some(blog_proto::Category {
                    id: row.get("id"),
                    name: row.get("name"),
                    is_del: row.get("is_del"),
                }),
            },
            None => GetCategoryReply { category: None },
        };
        Ok(tonic::Response::new(reply))
    }

    async fn list_category(
        &self,
        request: tonic::Request<ListCategoryRequest>,
    ) -> Result<tonic::Response<ListCategoryReply>, tonic::Status> {
        let ListCategoryRequest { name, is_del } = request.into_inner();
        let query = match name {
            Some(name) => {
                let name = format!("%{}%", name);
                match is_del {
                    Some(is_del) => {
                        sqlx::query(
                            "select id,name,is_del from categories where name ilike $1 and is_del=$2 order by id"
                        )
                        .bind(name.clone())
                        .bind(is_del)
                    }
                    None => sqlx::query("select id,name,is_del from categories where name ilike $1 order by id")
                    .bind(name),
                }
            }
            None => match is_del {
                Some(is_del) => {
                    sqlx::query("select id,name,is_del from categories where is_del=$1 order by id")
                        .bind(is_del)
                }
                None => sqlx::query("select id,name,is_del from categories order by id"),
            },
        };
        let rows = query
            .fetch_all(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;

        if rows.is_empty() {
            return Err(tonic::Status::not_found("没有符合条件的分类"));
        }
        let mut categories = Vec::with_capacity(rows.len());
        for row in rows {
            categories.push(blog_proto::Category {
                id: row.get("id"),
                name: row.get("name"),
                is_del: row.get("is_del"),
            });
        }
        let reply = ListCategoryReply { categories };
        Ok(tonic::Response::new(reply))
    }

    async fn toggle_category(
        &self,
        request: tonic::Request<ToggleCategoryRequest>,
    ) -> Result<Response<ToggleCategoryReply>, tonic::Status> {
        let ToggleCategoryRequest { id } = request.into_inner();
        let row =
            sqlx::query("update categories set is_del=(not is_del) where id=$1 returning is_del")
                .bind(id)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|err| tonic::Status::internal(err.to_string()))?;
        if let Some(row) = row {
            return Ok(tonic::Response::new(ToggleCategoryReply {
                id,
                is_del: row.get(0),
            }));
        }
        Err(tonic::Status::not_found("不存在的分类"))
    }
}
