use std::sync::Arc;

use blog_proto::{
    topic_service_server::TopicService, CreateTopicReply, CreateTopicRequest, EditTopicReply,
    EditTopicRequest, GetTopicReply, GetTopicRequest, ListTopicReply, ListTopicRequest,
    ToggleTopicReply, ToggleTopicRequest,
};
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use sqlx::{PgPool, Row};

pub struct Topic {
    pool: Arc<PgPool>,
}

impl Topic {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}

#[tonic::async_trait]
impl TopicService for Topic {
    async fn create_topic(
        &self,
        request: tonic::Request<CreateTopicRequest>,
    ) -> Result<tonic::Response<CreateTopicReply>, tonic::Status> {
        let CreateTopicRequest {
            title,
            category_id,
            content,
            summary,
        } = request.into_inner();
        let summary = match summary {
            Some(summary) => summary,
            None => get_summary(&content),
        };
        let row = sqlx::query("insert into topic (title,category_id,content,summary) values($1, $2, $3, $4) returning id")
            .bind(title)
            .bind(category_id).bind(content).bind(summary)
            .fetch_one(&*self.pool)
            .await.map_err(|err| tonic::Status::internal(err.to_string()))?;
        let reply = CreateTopicReply { id: row.get("id") };
        Ok(tonic::Response::new(reply))
    }
    async fn edit_topic(
        &self,
        request: tonic::Request<EditTopicRequest>,
    ) -> Result<tonic::Response<EditTopicReply>, tonic::Status> {
        let r = request.into_inner();
        let summary = match r.summary {
            Some(s) => s,
            None => get_summary(&r.content),
        };
        let rows_affected = sqlx::query(
            "update topic set title=$1,content=$2,summary=$3,category_id=$4 where id=$5",
        )
        .bind(r.title)
        .bind(r.content)
        .bind(summary)
        .bind(r.category_id)
        .bind(r.id)
        .execute(&*self.pool)
        .await
        .map_err(|err| tonic::Status::internal(err.to_string()))?
        .rows_affected();
        Ok(tonic::Response::new(EditTopicReply {
            id: r.id,
            ok: rows_affected > 0,
        }))
    }

    async fn toggle_topic(
        &self,
        request: tonic::Request<ToggleTopicRequest>,
    ) -> Result<tonic::Response<ToggleTopicReply>, tonic::Status> {
        let ToggleTopicRequest { id } = request.into_inner();
        let row = sqlx::query("update topics set is_del=(not is_del) where id=$1 returning is_del")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        if row.is_none() {
            return Err(tonic::Status::not_found("不存在的文章"));
        }
        Ok(tonic::Response::new(ToggleTopicReply {
            id,
            is_del: row.unwrap().get("is_del"),
        }))
    }

    async fn get_topic(
        &self,
        request: tonic::Request<GetTopicRequest>,
    ) -> Result<tonic::Response<GetTopicReply>, tonic::Status> {
        let GetTopicRequest {
            id,
            is_del,
            inc_hit,
        } = request.into_inner();

        let inc_hit = inc_hit.unwrap_or(false); // 增加点击量
        if inc_hit {
            sqlx::query("UPDATE topics SET hit=hit+1 WHERE id=$1")
                .bind(id)
                .execute(&*self.pool)
                .await
                .map_err(|err| tonic::Status::internal(err.to_string()))?;
        }

        let query = match is_del {
            Some(is_del) => sqlx::query("SELECT id,title,content,summary,is_del,category_id,dateline,hit FROM topics WHERE id=$1 AND is_del=$2")
            .bind(id).bind(is_del),
            None => sqlx::query("SELECT id,title,content,summary,is_del,category_id,dateline,hit FROM topics WHERE id=$1")
            .bind(id),
        };
        let row = query
            .fetch_optional(&*self.pool)
            .await
            .map_err(|err| tonic::Status::internal(err.to_string()))?;
        if row.is_none() {
            return Err(tonic::Status::not_found("不存在的文章"));
        }
        let row = row.unwrap();
        let dt: DateTime<Local> = row.get("dateline");
        let dateline = dt_conver(&dt);

        Ok(tonic::Response::new(GetTopicReply {
            topic: Some(blog_proto::Topic {
                id: row.get("id"),
                title: row.get("title"),
                category_id: row.get("category_id"),
                content: row.get("content"),
                summary: row.get("summary"),
                hit: row.get("hit"),
                is_del: row.get("is_del"),
                dateline,
            }),
        }))
    }

    async fn list_topic(
        &self,
        request: tonic::Request<ListTopicRequest>,
    ) -> Result<tonic::Response<ListTopicReply>, tonic::Status> {
        let ListTopicRequest {
            page,
            category_id,
            keyword,
            is_del,
            dateline_range,
        } = request.into_inner();

        let page = page.unwrap_or(0);
        let page_size = 30;
        let offset = page * page_size;
        let mut start = None;
        let mut end = None;
        if let Some(dr) = dateline_range {
            start = tm_cover(dr.start);
            end = tm_cover(dr.end);
        }
        let row = sqlx::query(
            r#"
            select count(*)
            from topics
            WHERE 1=1
                AND ($1::int IS NULL OR category_id = $1::int)
                AND ($2::text IS NULL OR title ILIKE CONCAT('%',$2::text,'%'))
                AND ($3::boolean IS NULL OR is_del = $3::boolean)
                AND (
                    ($4::TIMESTAMPTZ IS NULL OR $5::TIMESTAMPTZ IS NULL)
                    OR
                    (dateline BETWEEN $4::TIMESTAMPTZ AND $5::TIMESTAMPTZ)
                )"#,
        )
        .bind(&category_id)
        .bind(&keyword)
        .bind(is_del)
        .bind(start)
        .bind(end)
        .fetch_one(&*self.pool)
        .await
        .map_err(|err| tonic::Status::internal(err.to_string()))?;

        let record_total: i64 = row.get(0);
        let page_totoal = f64::ceil(record_total as f64 / page_size as f64) as i64;

        let rows = sqlx::query(
            r#"
            SELECT 
                id,title,content,summary,is_del,category_id,dateline,hit FROM topics
            WHERE 1=1
                AND ($3::int IS NULL OR category_id = $3::int)
                AND ($4::text IS NULL OR title ILIKE CONCAT('%',$4::text,'%'))
                AND ($5::boolean IS NULL OR is_del = $5::boolean)
                AND (
                    ($6::TIMESTAMPTZ IS NULL OR $7::TIMESTAMPTZ IS NULL)
                    OR
                    (dateline BETWEEN $6::TIMESTAMPTZ AND $7::TIMESTAMPTZ)
            )
            ORDER BY 
                id DESC
            LIMIT 
                $1
            OFFSET
                $2
            "#,
        )
        .bind(page_size)
        .bind(offset)
        .bind(&category_id)
        .bind(&keyword)
        .bind(&is_del)
        .bind(&start)
        .bind(&end)
        .fetch_all(&*self.pool)
        .await
        .map_err(|err| tonic::Status::internal(err.to_string()))?;

        let mut topics = Vec::with_capacity(rows.len());
        for row in rows {
            let dt: DateTime<Local> = row.get("dateline");
            let dateline = dt_conver(&dt);
            topics.push(blog_proto::Topic {
                id: row.get("id"),
                title: row.get("title"),
                category_id: row.get("category_id"),
                content: row.get("content"),
                summary: row.get("summary"),
                hit: row.get("hit"),
                is_del: row.get("is_del"),
                dateline,
            });
        }

        Ok(tonic::Response::new(ListTopicReply {
            page,
            page_size,
            page_totoal,
            record_total,
            topics,
        }))
    }
}

fn get_summary(content: &str) -> String {
    if content.len() <= 255 {
        return String::from(content);
    }
    content.chars().into_iter().take(255).collect()
}

fn dt_conver(dt: &DateTime<Local>) -> Option<prost_types::Timestamp> {
    if let Ok(dt) = prost_types::Timestamp::date_time(
        dt.year().into(),
        dt.month() as u8,
        dt.day() as u8,
        dt.hour() as u8,
        dt.minute() as u8,
        dt.second() as u8,
    ) {
        Some(dt)
    } else {
        None
    }
}

fn tm_cover(tm: Option<prost_types::Timestamp>) -> Option<DateTime<Local>> {
    match tm {
        Some(tm) => Some(Local.timestamp_opt(tm.seconds, 0).unwrap()),
        None => None,
    }
}
