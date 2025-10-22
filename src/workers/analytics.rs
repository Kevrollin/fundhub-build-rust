use std::time::Duration;
use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, error, warn};
use chrono::{Utc, Duration as ChronoDuration};
use sqlx::types::BigDecimal;
use num_traits::cast::ToPrimitive;

pub struct AnalyticsWorker {
    pool: PgPool,
}

impl AnalyticsWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start(self) -> Result<()> {
        info!("Starting analytics worker...");
        
        // Real-time analytics collection (every 5 minutes)
        let pool_clone = self.pool.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::collect_realtime_analytics(&pool_clone).await {
                    error!("Error collecting real-time analytics: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(300)).await;
            }
        });

        // Daily analytics aggregation (every hour)
        let pool_clone2 = self.pool.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::aggregate_daily_analytics(&pool_clone2).await {
                    error!("Error aggregating daily analytics: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });

        // Weekly analytics summary (every 6 hours)
        let pool_clone3 = self.pool.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::generate_weekly_summary(&pool_clone3).await {
                    error!("Error generating weekly summary: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(21600)).await;
            }
        });

        Ok(())
    }

    async fn collect_realtime_analytics(pool: &PgPool) -> Result<()> {
        // Update project analytics
        Self::update_project_analytics(pool).await?;
        
        // Update student analytics
        Self::update_student_analytics(pool).await?;
        
        // Update campaign analytics
        Self::update_campaign_analytics(pool).await?;
        
        // Update platform metrics
        Self::update_platform_metrics(pool).await?;

        Ok(())
    }

    async fn aggregate_daily_analytics(pool: &PgPool) -> Result<()> {
        let today = Utc::now().date_naive();
        let yesterday = today - ChronoDuration::days(1);

        // Daily donation trends
        Self::aggregate_donation_trends(pool, yesterday).await?;
        
        // Daily user activity
        Self::aggregate_user_activity(pool, yesterday).await?;
        
        // Daily project performance
        Self::aggregate_project_performance(pool, yesterday).await?;

        info!("Daily analytics aggregated for {}", yesterday);
        Ok(())
    }

    async fn generate_weekly_summary(pool: &PgPool) -> Result<()> {
        let week_start = Utc::now().date_naive() - ChronoDuration::days(7);
        
        // Weekly platform growth
        Self::calculate_weekly_growth(pool, week_start).await?;
        
        // Weekly top performers
        Self::calculate_weekly_top_performers(pool, week_start).await?;
        
        // Weekly campaign effectiveness
        Self::calculate_weekly_campaign_effectiveness(pool, week_start).await?;

        info!("Weekly summary generated for week starting {}", week_start);
        Ok(())
    }

    async fn update_project_analytics(pool: &PgPool) -> Result<()> {
        // Total donations per project
        let rows = sqlx::query!(
            r#"SELECT project_id, SUM(amount) as total FROM donations WHERE status = 'confirmed' GROUP BY project_id"#
        ).fetch_all(pool).await?;
        
        for r in rows {
            let _ = sqlx::query!(
                r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                    VALUES ('project', $1, 'total_donations', $2, NOW())
                    ON CONFLICT (entity_type, entity_id, metric)
                    DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
                r.project_id,
                r.total.unwrap_or_default().to_f64().unwrap_or(0.0)
            ).execute(pool).await;
        }

        // Donation count per project
        let count_rows = sqlx::query!(
            r#"SELECT project_id, COUNT(*) as count FROM donations WHERE status = 'confirmed' GROUP BY project_id"#
        ).fetch_all(pool).await?;
        
        for r in count_rows {
            let _ = sqlx::query!(
                r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                    VALUES ('project', $1, 'donation_count', $2, NOW())
                    ON CONFLICT (entity_type, entity_id, metric)
                    DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
                r.project_id,
                r.count.unwrap_or(0) as f64
            ).execute(pool).await;
        }

        Ok(())
    }

    async fn update_student_analytics(pool: &PgPool) -> Result<()> {
        // Total donations per student (via projects)
        let rows = sqlx::query!(
            r#"SELECT p.student_id as sid, SUM(d.amount) as total
                FROM donations d
                JOIN projects p ON p.id = d.project_id
                WHERE d.status = 'confirmed'
                GROUP BY p.student_id"#
        ).fetch_all(pool).await?;
        
        for r in rows {
            let _ = sqlx::query!(
                r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                    VALUES ('student', $1, 'total_donations', $2, NOW())
                    ON CONFLICT (entity_type, entity_id, metric)
                    DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
                r.sid,
                r.total.unwrap_or_default().to_f64().unwrap_or(0.0)
            ).execute(pool).await;
        }

        // Project count per student
        let project_rows = sqlx::query!(
            r#"SELECT student_id, COUNT(*) as count FROM projects GROUP BY student_id"#
        ).fetch_all(pool).await?;
        
        for r in project_rows {
            let _ = sqlx::query!(
                r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                    VALUES ('student', $1, 'project_count', $2, NOW())
                    ON CONFLICT (entity_type, entity_id, metric)
                    DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
                r.student_id,
                r.count.unwrap_or(0) as f64
            ).execute(pool).await;
        }

        Ok(())
    }

    async fn update_campaign_analytics(pool: &PgPool) -> Result<()> {
        // Campaign distribution amounts
        let rows = sqlx::query!(
            r#"SELECT campaign_id, SUM(amount) as total FROM campaign_distributions GROUP BY campaign_id"#
        ).fetch_all(pool).await?;
        
        for r in rows {
            let _ = sqlx::query!(
                r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                    VALUES ('campaign', $1, 'distributed_amount', $2, NOW())
                    ON CONFLICT (entity_type, entity_id, metric)
                    DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
                r.campaign_id,
                r.total.unwrap_or_default().to_f64().unwrap_or(0.0)
            ).execute(pool).await;
        }

        // Campaign recipient count
        let recipient_rows = sqlx::query!(
            r#"SELECT campaign_id, COUNT(DISTINCT recipient_id) as count FROM campaign_distributions GROUP BY campaign_id"#
        ).fetch_all(pool).await?;
        
        for r in recipient_rows {
            let _ = sqlx::query!(
                r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                    VALUES ('campaign', $1, 'recipient_count', $2, NOW())
                    ON CONFLICT (entity_type, entity_id, metric)
                    DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
                r.campaign_id,
                r.count.unwrap_or(0) as f64
            ).execute(pool).await;
        }

        Ok(())
    }

    async fn update_platform_metrics(pool: &PgPool) -> Result<()> {
        // Total platform donations
        let total_donations = sqlx::query!(
            r#"SELECT COALESCE(SUM(amount), 0) as total FROM donations WHERE status = 'confirmed'"#
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                VALUES ('platform', '00000000-0000-0000-0000-000000000000', 'total_donations', $1, NOW())
                ON CONFLICT (entity_type, entity_id, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            total_donations.total.unwrap_or(BigDecimal::from(0)).to_f64().unwrap_or(0.0)
        ).execute(pool).await;

        // Total users
        let total_users = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM users"#
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                VALUES ('platform', '00000000-0000-0000-0000-000000000000', 'total_users', $1, NOW())
                ON CONFLICT (entity_type, entity_id, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            total_users.count.unwrap_or(0) as f64
        ).execute(pool).await;

        Ok(())
    }

    async fn aggregate_donation_trends(pool: &PgPool, date: chrono::NaiveDate) -> Result<()> {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        let daily_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as donation_count,
                SUM(amount) as total_amount
            FROM donations 
            WHERE status = 'confirmed' 
                AND created_at >= $1 
                AND created_at <= $2
            "#,
            start_of_day, end_of_day
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO daily_analytics (date, metric, value, created_at)
                VALUES ($1, 'donation_count', $2, NOW())
                ON CONFLICT (date, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            date,
            daily_stats.donation_count.unwrap_or(0) as f64
        ).execute(pool).await;

        let _ = sqlx::query!(
            r#"INSERT INTO daily_analytics (date, metric, value, created_at)
                VALUES ($1, 'donation_amount', $2, NOW())
                ON CONFLICT (date, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            date,
            daily_stats.total_amount.unwrap_or(BigDecimal::from(0)).to_f64().unwrap_or(0.0)
        ).execute(pool).await;

        Ok(())
    }

    async fn aggregate_user_activity(pool: &PgPool, date: chrono::NaiveDate) -> Result<()> {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        // New user registrations
        let new_users = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM users WHERE created_at >= $1 AND created_at <= $2"#,
            start_of_day, end_of_day
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO daily_analytics (date, metric, value, created_at)
                VALUES ($1, 'new_users', $2, NOW())
                ON CONFLICT (date, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            date,
            new_users.count.unwrap_or(0) as f64
        ).execute(pool).await;

        // New student verifications
        let new_students = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM students WHERE verification_status = 'verified' AND created_at >= $1 AND created_at <= $2"#,
            start_of_day, end_of_day
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO daily_analytics (date, metric, value, created_at)
                VALUES ($1, 'new_students', $2, NOW())
                ON CONFLICT (date, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            date,
            new_students.count.unwrap_or(0) as f64
        ).execute(pool).await;

        Ok(())
    }

    async fn aggregate_project_performance(pool: &PgPool, date: chrono::NaiveDate) -> Result<()> {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        // New projects created
        let new_projects = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM projects WHERE created_at >= $1 AND created_at <= $2"#,
            start_of_day, end_of_day
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO daily_analytics (date, metric, value, created_at)
                VALUES ($1, 'new_projects', $2, NOW())
                ON CONFLICT (date, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            date,
            new_projects.count.unwrap_or(0) as f64
        ).execute(pool).await;

        Ok(())
    }

    async fn calculate_weekly_growth(pool: &PgPool, week_start: chrono::NaiveDate) -> Result<()> {
        let week_end = (week_start + ChronoDuration::days(7)).and_hms_opt(23, 59, 59).unwrap().and_utc();
        let week_start_dt = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Calculate growth metrics
        let growth_metrics = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT u.id) as new_users,
                COUNT(DISTINCT s.id) as new_students,
                COUNT(DISTINCT p.id) as new_projects,
                SUM(d.amount) as total_donations
            FROM users u
            LEFT JOIN students s ON s.user_id = u.id AND s.created_at >= $1 AND s.created_at <= $2
            LEFT JOIN projects p ON p.student_id = s.id AND p.created_at >= $1 AND p.created_at <= $2
            LEFT JOIN donations d ON d.project_id = p.id AND d.status = 'confirmed' AND d.created_at >= $1 AND d.created_at <= $2
            WHERE u.created_at >= $1 AND u.created_at <= $2
            "#,
            week_start_dt, week_end
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO weekly_analytics (week_start, metric, value, created_at)
                VALUES ($1, 'new_users', $2, NOW())
                ON CONFLICT (week_start, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            week_start,
            growth_metrics.new_users.unwrap_or(0) as f64
        ).execute(pool).await;

        Ok(())
    }

    async fn calculate_weekly_top_performers(pool: &PgPool, week_start: chrono::NaiveDate) -> Result<()> {
        let week_end = (week_start + ChronoDuration::days(7)).and_hms_opt(23, 59, 59).unwrap().and_utc();
        let week_start_dt = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Top performing projects this week
        let top_projects = sqlx::query!(
            r#"
            SELECT p.id, SUM(d.amount) as total
            FROM projects p
            JOIN donations d ON p.id = d.project_id
            WHERE d.status = 'confirmed' 
                AND d.created_at >= $1 
                AND d.created_at <= $2
            GROUP BY p.id
            ORDER BY total DESC
            LIMIT 5
            "#,
            week_start_dt, week_end
        ).fetch_all(pool).await?;

        for (i, project) in top_projects.iter().enumerate() {
            let _ = sqlx::query!(
                r#"INSERT INTO weekly_analytics (week_start, metric, value, created_at)
                    VALUES ($1, $2, $3, NOW())
                    ON CONFLICT (week_start, metric)
                    DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
                week_start,
                format!("top_project_{}", i + 1),
                project.total.clone().unwrap_or(BigDecimal::from(0)).to_f64().unwrap_or(0.0)
            ).execute(pool).await;
        }

        Ok(())
    }

    async fn calculate_weekly_campaign_effectiveness(pool: &PgPool, week_start: chrono::NaiveDate) -> Result<()> {
        let week_end = (week_start + ChronoDuration::days(7)).and_hms_opt(23, 59, 59).unwrap().and_utc();
        let week_start_dt = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Campaign effectiveness metrics
        let campaign_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT c.id) as active_campaigns,
                SUM(c.reward_pool_xlm) as total_reward_pool,
                SUM(cd.amount) as distributed_amount
            FROM campaigns c
            LEFT JOIN campaign_distributions cd ON c.id = cd.campaign_id
                AND cd.created_at >= $1 
                AND cd.created_at <= $2
            WHERE c.status = 'active'
                AND c.created_at <= $2
            "#,
            week_start_dt, week_end
        ).fetch_one(pool).await?;

        let _ = sqlx::query!(
            r#"INSERT INTO weekly_analytics (week_start, metric, value, created_at)
                VALUES ($1, 'active_campaigns', $2, NOW())
                ON CONFLICT (week_start, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            week_start,
            campaign_stats.active_campaigns.unwrap_or(0) as f64
        ).execute(pool).await;

        Ok(())
    }
}
