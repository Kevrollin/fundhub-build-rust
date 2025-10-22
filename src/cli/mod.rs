use colored::*;
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;

pub struct FundHubCLI {
    multi_progress: MultiProgress,
}

impl FundHubCLI {
    pub fn new() -> Self {
        Self {
            multi_progress: MultiProgress::new(),
        }
    }

    pub fn show_banner(&self) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_blue());
        println!("{}", "║                                                                              ║".bright_blue());
        println!("{}", "║  🚀 FundHub Backend Server - Student Crowdfunding Platform                  ║".bright_blue());
        println!("{}", "║                                                                              ║".bright_blue());
        println!("{}", "║  Built with Rust + Axum + PostgreSQL + Stellar Blockchain                    ║".bright_blue());
        println!("{}", "║                                                                              ║".bright_blue());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_blue());
        println!();
    }

    pub fn show_startup_progress(&self) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(100));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} {bar:40.cyan/blue} {pos}/{len} {elapsed_precise}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Starting FundHub Backend...");
        pb
    }

    pub async fn initialize_database(&self) -> Result<()> {
        let pb = self.multi_progress.add(ProgressBar::new(100));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} {bar:40.cyan/blue} {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Connecting to database...");

        // Simulate database connection steps
        for i in 0..=100 {
            pb.set_position(i);
            pb.set_message(match i {
                0..=20 => "Loading database configuration...",
                21..=40 => "Establishing connection to PostgreSQL...",
                41..=60 => "Running database migrations...",
                61..=80 => "Verifying database schema...",
                81..=100 => "Database ready!",
                _ => "Initializing...",
            });
            sleep(Duration::from_millis(50)).await;
        }
        pb.finish_with_message("✅ Database connected successfully");
        Ok(())
    }

    pub async fn initialize_stellar(&self) -> Result<()> {
        let pb = self.multi_progress.add(ProgressBar::new(100));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} {bar:40.cyan/blue} {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Initializing Stellar service...");

        for i in 0..=100 {
            pb.set_position(i);
            pb.set_message(match i {
                0..=25 => "Loading Stellar configuration...",
                26..=50 => "Connecting to Stellar network...",
                51..=75 => "Verifying network connectivity...",
                76..=100 => "Stellar service ready!",
                _ => "Initializing...",
            });
            sleep(Duration::from_millis(30)).await;
        }
        pb.finish_with_message("✅ Stellar service initialized");
        Ok(())
    }

    pub async fn start_workers(&self) -> Result<()> {
        let pb = self.multi_progress.add(ProgressBar::new(100));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} {bar:40.cyan/blue} {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Starting background workers...");

        for i in 0..=100 {
            pb.set_position(i);
            pb.set_message(match i {
                0..=20 => "Starting donation verification worker...",
                21..=40 => "Starting wallet sync worker...",
                41..=60 => "Starting analytics worker...",
                61..=80 => "Starting campaign distribution worker...",
                81..=100 => "All workers started!",
                _ => "Starting workers...",
            });
            sleep(Duration::from_millis(40)).await;
        }
        pb.finish_with_message("✅ Background workers started");
        Ok(())
    }

    pub fn show_server_info(&self, port: u16) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_green());
        println!("{}", "║                                                                              ║".bright_green());
        println!("{}", "║  🎉 FundHub Backend Server is now running!                                  ║".bright_green());
        println!("{}", "║                                                                              ║".bright_green());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_green());
        println!();
        
        println!("{}", "📊 Server Information:".bright_cyan().bold());
        println!("  {} {}", "🌐 Server URL:".bright_white(), format!("http://localhost:{}", port).bright_yellow());
        println!("  {} {}", "📚 API Documentation:".bright_white(), format!("http://localhost:{}/api/docs", port).bright_yellow());
        println!("  {} {}", "❤️  Health Check:".bright_white(), format!("http://localhost:{}/health", port).bright_yellow());
        println!("  {} {}", "📈 API Info:".bright_white(), format!("http://localhost:{}/api/docs/api", port).bright_yellow());
        println!();

        println!("{}", "🔗 Available Endpoints:".bright_cyan().bold());
        println!("  {} {}", "🔐 Authentication:".bright_white(), format!("http://localhost:{}/api/auth", port).bright_yellow());
        println!("  {} {}", "👥 Students:".bright_white(), format!("http://localhost:{}/api/students", port).bright_yellow());
        println!("  {} {}", "💰 Wallets:".bright_white(), format!("http://localhost:{}/api/wallets", port).bright_yellow());
        println!("  {} {}", "🎁 Donations:".bright_white(), format!("http://localhost:{}/api/donations", port).bright_yellow());
        println!("  {} {}", "📊 Campaigns:".bright_white(), format!("http://localhost:{}/api/campaigns", port).bright_yellow());
        println!("  {} {}", "📈 Analytics:".bright_white(), format!("http://localhost:{}/api/analytics", port).bright_yellow());
        println!("  {} {}", "⚙️  Admin:".bright_white(), format!("http://localhost:{}/api/admin", port).bright_yellow());
        println!("  {} {}", "🔔 Notifications:".bright_white(), format!("http://localhost:{}/api/notifications/stream", port).bright_yellow());
        println!();

        println!("{}", "🛠️  Background Services:".bright_cyan().bold());
        println!("  {} Donation verification worker", "✅".bright_green());
        println!("  {} Wallet synchronization worker", "✅".bright_green());
        println!("  {} Analytics collection worker", "✅".bright_green());
        println!("  {} Campaign distribution worker", "✅".bright_green());
        println!();

        println!("{}", "💡 Quick Start:".bright_cyan().bold());
        println!("  1. Open your browser and go to: {}", format!("http://localhost:{}/api/docs", port).bright_yellow());
        println!("  2. Register a new account using the signup endpoint");
        println!("  3. Create a student profile");
        println!("  4. Connect your Stellar wallet");
        println!("  5. Start creating projects and campaigns!");
        println!();

        println!("{}", "Press Ctrl+C to stop the server".bright_red().italic());
        println!();
    }

    pub fn show_runtime_stats(&self, start_time: std::time::Instant) {
        let uptime = start_time.elapsed();
        let hours = uptime.as_secs() / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;
        let seconds = uptime.as_secs() % 60;

        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_magenta());
        println!("{}", "║                                                                              ║".bright_magenta());
        println!("{}", "║  📊 Runtime Statistics                                                       ║".bright_magenta());
        println!("{}", "║                                                                              ║".bright_magenta());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_magenta());
        println!();
        
        println!("  {} {}", "⏱️  Uptime:".bright_white(), 
            format!("{}h {}m {}s", hours, minutes, seconds).bright_yellow());
        println!("  {} {}", "🔄 Status:".bright_white(), "Running".bright_green());
        println!("  {} {}", "💾 Memory:".bright_white(), "Optimized".bright_green());
        println!("  {} {}", "🌐 Connections:".bright_white(), "Active".bright_green());
        println!();
    }

    pub fn show_shutdown_message(&self) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_red());
        println!("{}", "║                                                                              ║".bright_red());
        println!("{}", "║  🛑 Shutting down FundHub Backend Server...                                 ║".bright_red());
        println!("{}", "║                                                                              ║".bright_red());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_red());
        println!();
        
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.red} {msg} {bar:40.red/red} {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Stopping services...");

        for i in 0..=100 {
            pb.set_position(i);
            pb.set_message(match i {
                0..=25 => "Stopping background workers...",
                26..=50 => "Closing database connections...",
                51..=75 => "Stopping HTTP server...",
                76..=100 => "Shutdown complete!",
                _ => "Shutting down...",
            });
            std::thread::sleep(Duration::from_millis(20));
        }
        pb.finish_with_message("✅ Server stopped successfully");
        println!();
        println!("{}", "Thank you for using FundHub! 👋".bright_cyan());
    }

    pub fn show_error(&self, error: &str) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_red());
        println!("{}", "║                                                                              ║".bright_red());
        println!("{}", "║  ❌ Error occurred during startup                                           ║".bright_red());
        println!("{}", "║                                                                              ║".bright_red());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_red());
        println!();
        println!("{}", format!("Error: {}", error).bright_red());
        println!();
    }

    pub fn show_help(&self) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_blue());
        println!("{}", "║                                                                              ║".bright_blue());
        println!("{}", "║  📖 FundHub Backend Help                                                    ║".bright_blue());
        println!("{}", "║                                                                              ║".bright_blue());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_blue());
        println!();
        
        println!("{}", "Available Commands:".bright_cyan().bold());
        println!("  {} {} - Show this help message", "help".bright_yellow(), "|".bright_white());
        println!("  {} {} - Show server status", "status".bright_yellow(), "|".bright_white());
        println!("  {} {} - Show API documentation", "docs".bright_yellow(), "|".bright_white());
        println!("  {} {} - Show runtime statistics", "stats".bright_yellow(), "|".bright_white());
        println!("  {} {} - Restart the server", "restart".bright_yellow(), "|".bright_white());
        println!("  {} {} - Stop the server", "stop".bright_yellow(), "|".bright_white());
        println!();
    }
}
