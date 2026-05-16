fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use chrono::{Local, NaiveDateTime};
    use futures::TryStreamExt;
    use sqlx::{
        Connection, Error, PgConnection, Pool, Postgres, Row,
        postgres::{PgPoolOptions, PgRow},
        prelude::FromRow,
    };

    #[derive(FromRow, Debug)]
    #[allow(dead_code)]
    struct Category {
        id: String,
        name: String,
        description: String,
    }

    #[tokio::test]
    async fn connection_test() -> Result<(), Error> {
        let url = "postgres://postgres:qwerty1234@127.0.0.1:5432/rust_database";

        let connection: PgConnection = PgConnection::connect(url).await?;

        connection.close().await?;

        Ok(())
    }

    async fn get_pool() -> Result<Pool<Postgres>, Error> {
        let url = "postgres://postgres:qwerty1234@127.0.0.1:5432/rust_database";
        PgPoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(60))
            .connect(url)
            .await
    }

    use tokio::sync::OnceCell;

    static POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();

    async fn get_pool_once() -> &'static Pool<Postgres> {
        let url = "postgres://postgres:qwerty1234@127.0.0.1:5432/rust_database";

        POOL.get_or_init(|| async {
            PgPoolOptions::new()
                .max_connections(5)
                .connect(url)
                .await
                .unwrap()
        })
        .await
    }

    #[tokio::test]
    async fn pool_connection_test() -> Result<(), Error> {
        let pool = get_pool().await?;
        pool.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn execute_test() -> Result<(), Error> {
        let pool = get_pool().await?;
        sqlx::query("insert into category(id, name, description) values('A', 'Contoh', 'Contoh')")
            .execute(&pool)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn prepare_statement_test() -> Result<(), Error> {
        let pool = get_pool().await?;
        sqlx::query("insert into category(id, name, description) values($1, $2, $3)")
            .bind("C")
            .bind("Contoh C")
            .bind("Deskripsi C")
            .execute(&pool)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn fetch_optional_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();
        let result: Option<PgRow> = sqlx::query("select * from category where id = $1")
            .bind("C")
            .fetch_optional(&pool)
            .await?;

        if let Some(row) = result {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let description: String = row.get("description");
            println!("id: {}, name: {}, description: {}", id, name, description);
        } else {
            println!("Data Not Found!!!");
        }

        Ok(())
    }

    #[tokio::test]
    async fn fetch_one_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();
        let result: PgRow = sqlx::query("select * from category where id = $1")
            .bind("B")
            .fetch_one(&pool)
            .await?;

        let id: String = result.get("id");
        let name: String = result.get("name");
        let description: String = result.get("description");
        println!("id: {}, name: {}, description: {}", id, name, description);

        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();

        let results: Vec<PgRow> = sqlx::query("select * from category")
            .fetch_all(&pool)
            .await?;

        for result in results {
            let id: String = result.get("id");
            let name: String = result.get("name");
            let description: String = result.get("description");
            println!("id: {}, name: {}, description: {}", id, name, description);
        }

        Ok(())
    }

    #[tokio::test]
    async fn fetch_stream_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();
        let mut results = sqlx::query("select * from category").fetch(&pool);

        while let Some(row) = results.try_next().await? {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let description: String = row.get("description");
            println!("id: {}, name: {}, description: {}", id, name, description);
        }

        Ok(())
    }

    #[tokio::test]
    async fn result_mapping_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();

        let results: Vec<Category> = sqlx::query("select * from category")
            .map(|row: PgRow| Category {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
            })
            .fetch_all(&pool)
            .await?;

        for result in results {
            println!("{:?}", result);
        }

        Ok(())
    }

    #[tokio::test]
    async fn result_map_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();

        let results: Vec<Category> = sqlx::query_as("select * from category")
            .fetch_all(&pool)
            .await?;

        for result in results {
            println!("{:?}", result);
        }

        Ok(())
    }

    #[derive(FromRow, Debug)]
    #[allow(dead_code)]
    struct Brand {
        id: String,
        name: String,
        description: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    }

    #[tokio::test]
    async fn insert_brand_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();
        sqlx::query("insert into brand(id, name, description, created_at, updated_at) values($1, $2, $3, $4, $5)")
            .bind("A")
            .bind("Brand A")
            .bind("Description Brand A")
            .bind(Local::now().naive_local())
            .bind(Local::now().naive_local())
            .execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn insert_brand_b_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();
        sqlx::query("insert into brand(id, name, description, created_at, updated_at) values($1, $2, $3, $4, $5)")
            .bind("B")
            .bind("Brand B")
            .bind("Description Brand B")
            .bind(Local::now().naive_local())
            .bind(Local::now().naive_local())
            .execute(&pool).await?;

        Ok(())
    }

    #[tokio::test]
    async fn resultmap_brand_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();

        let results: Vec<Brand> = sqlx::query_as("select * from brand")
            .fetch_all(&pool)
            .await?;

        for result in results {
            println!("{:?}", result);
        }

        Ok(())
    }

    #[tokio::test]
    async fn insert_brand_c_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();
        let mut tx = pool.begin().await?;
        sqlx::query("insert into brand(id, name, description, created_at, updated_at) values($1, $2, $3, $4, $5)")
            .bind("C")
            .bind("Brand C")
            .bind("Description Brand C")
            .bind(Local::now().naive_local())
            .bind(Local::now().naive_local())
            .execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    #[tokio::test]
    async fn insert_brand_d_test() -> Result<(), Error> {
        let pool: Pool<Postgres> = get_pool_once().await.clone();

        let mut tx = pool.begin().await?;

        sqlx::query("insert into brand(id, name, description, created_at, updated_at) values($1, $2, $3, $4, $5)")
            .bind("D")
            .bind("Brand D")
            .bind("Description Brand D")
            .bind(Local::now().naive_local())
            .bind(Local::now().naive_local())
            .execute(&mut *tx).await?;

        sqlx::query("insert into brand(id, name, description, created_at, updated_at) values($1, $2, $3, $4, $5)")
            .bind("E")
            .bind("Brand E")
            .bind("Description Brand E")
            .bind(Local::now().naive_local())
            .bind(Local::now().naive_local())
            .execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }
}
