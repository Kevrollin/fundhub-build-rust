use axum::{extract::State, Json, http::StatusCode, response::Html};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiInfo {
    name: String,
    version: String,
    description: String,
    base_url: String,
    endpoints: Vec<EndpointInfo>,
}

#[derive(Serialize)]
pub struct EndpointInfo {
    method: String,
    path: String,
    description: String,
    category: String,
    auth_required: bool,
}

pub async fn api_info() -> Result<Json<ApiInfo>, StatusCode> {
    let endpoints = vec![
        // Authentication
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/auth/signup".to_string(),
            description: "Register a new user account".to_string(),
            category: "Authentication".to_string(),
            auth_required: false,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/auth/login".to_string(),
            description: "Login with email and password".to_string(),
            category: "Authentication".to_string(),
            auth_required: false,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/auth/logout".to_string(),
            description: "Logout user".to_string(),
            category: "Authentication".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/auth/profile/:user_id".to_string(),
            description: "Get user profile".to_string(),
            category: "Authentication".to_string(),
            auth_required: true,
        },
        
        // Students
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/students".to_string(),
            description: "Create a new student profile".to_string(),
            category: "Students".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/students/:id".to_string(),
            description: "Get student profile by ID".to_string(),
            category: "Students".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT".to_string(),
            path: "/api/students/:id".to_string(),
            description: "Update student profile".to_string(),
            category: "Students".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/students".to_string(),
            description: "List all students".to_string(),
            category: "Students".to_string(),
            auth_required: true,
        },
        
        // Wallets
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/wallets/connect".to_string(),
            description: "Connect a Stellar wallet".to_string(),
            category: "Wallets".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/wallets/balance/:wallet_id".to_string(),
            description: "Get wallet balance".to_string(),
            category: "Wallets".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/wallets/transactions/:wallet_id".to_string(),
            description: "Get wallet transactions".to_string(),
            category: "Wallets".to_string(),
            auth_required: true,
        },
        
        // Donations
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/donations/initiate".to_string(),
            description: "Initiate a donation".to_string(),
            category: "Donations".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/donations/verify".to_string(),
            description: "Verify a donation".to_string(),
            category: "Donations".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/donations/:project_id".to_string(),
            description: "Get donations for a project".to_string(),
            category: "Donations".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/donations/student/:student_id".to_string(),
            description: "Get donations by a student".to_string(),
            category: "Donations".to_string(),
            auth_required: true,
        },
        
        // Campaigns
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/campaigns/create".to_string(),
            description: "Create a new campaign".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/campaigns/execute".to_string(),
            description: "Execute a campaign".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/campaigns/active".to_string(),
            description: "List active campaigns".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/campaigns/stats".to_string(),
            description: "Get campaign statistics".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/campaigns/:id".to_string(),
            description: "Get campaign by ID".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "PUT".to_string(),
            path: "/api/campaigns/:id".to_string(),
            description: "Update campaign".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "DELETE".to_string(),
            path: "/api/campaigns/:id".to_string(),
            description: "Delete campaign".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/campaigns/:id/pause".to_string(),
            description: "Pause campaign".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/campaigns/:id/resume".to_string(),
            description: "Resume campaign".to_string(),
            category: "Campaigns".to_string(),
            auth_required: true,
        },
        
        // Analytics
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/analytics/platform/stats".to_string(),
            description: "Get platform statistics".to_string(),
            category: "Analytics".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/analytics/projects/top".to_string(),
            description: "Get top performing projects".to_string(),
            category: "Analytics".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/analytics/students/top".to_string(),
            description: "Get top performing students".to_string(),
            category: "Analytics".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/analytics/campaigns/performance".to_string(),
            description: "Get campaign performance metrics".to_string(),
            category: "Analytics".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/analytics/donations/trends".to_string(),
            description: "Get donation trends".to_string(),
            category: "Analytics".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/analytics/projects/:id".to_string(),
            description: "Get project-specific analytics".to_string(),
            category: "Analytics".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/analytics/students/:id".to_string(),
            description: "Get student-specific analytics".to_string(),
            category: "Analytics".to_string(),
            auth_required: true,
        },
        
        // Admin
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/admin/students".to_string(),
            description: "List all students (admin only)".to_string(),
            category: "Admin".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/admin/verify-student".to_string(),
            description: "Verify a student (admin only)".to_string(),
            category: "Admin".to_string(),
            auth_required: true,
        },
        EndpointInfo {
            method: "POST".to_string(),
            path: "/api/admin/fund-student".to_string(),
            description: "Fund a student (admin only)".to_string(),
            category: "Admin".to_string(),
            auth_required: true,
        },
        
        // Notifications
        EndpointInfo {
            method: "GET".to_string(),
            path: "/api/notifications/stream".to_string(),
            description: "Stream real-time notifications (SSE)".to_string(),
            category: "Notifications".to_string(),
            auth_required: true,
        },
    ];

    Ok(Json(ApiInfo {
        name: "FundHub API".to_string(),
        version: "1.0.0".to_string(),
        description: "A comprehensive crowdfunding platform for students with Stellar blockchain integration".to_string(),
        base_url: "http://localhost:3000".to_string(),
        endpoints,
    }))
}

pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "service": "FundHub API",
        "uptime": "running"
    })))
}

pub async fn docs_html() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FundHub API Documentation</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }
        
        .header {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 3rem 2rem;
            margin-bottom: 2rem;
            text-align: center;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
        }
        
        .header h1 {
            font-size: 3rem;
            background: linear-gradient(135deg, #667eea, #764ba2);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            margin-bottom: 1rem;
        }
        
        .header p {
            font-size: 1.2rem;
            color: #666;
            margin-bottom: 2rem;
        }
        
        .nav {
            display: flex;
            gap: 1rem;
            margin-bottom: 2rem;
            flex-wrap: wrap;
            justify-content: center;
        }
        
        .nav a {
            background: rgba(255, 255, 255, 0.9);
            color: #667eea;
            padding: 1rem 2rem;
            text-decoration: none;
            border-radius: 50px;
            font-weight: 600;
            box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
            transition: all 0.3s ease;
            border: 2px solid transparent;
        }
        
        .nav a:hover {
            background: #667eea;
            color: white;
            transform: translateY(-3px);
            box-shadow: 0 8px 25px rgba(102, 126, 234, 0.3);
        }
        
        .section {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            padding: 2rem;
            border-radius: 20px;
            margin-bottom: 2rem;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
        }
        
        .section h2 {
            color: #667eea;
            border-bottom: 3px solid #667eea;
            padding-bottom: 1rem;
            margin-bottom: 2rem;
            font-size: 2rem;
        }
        
        .endpoint {
            background: #f8f9fa;
            padding: 1.5rem;
            margin: 1rem 0;
            border-radius: 15px;
            border-left: 5px solid #667eea;
            transition: all 0.3s ease;
            position: relative;
            overflow: hidden;
        }
        
        .endpoint:hover {
            transform: translateX(10px);
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
        }
        
        .endpoint-header {
            display: flex;
            align-items: center;
            margin-bottom: 1rem;
            flex-wrap: wrap;
            gap: 1rem;
        }
        
        .method {
            display: inline-block;
            padding: 0.5rem 1rem;
            border-radius: 25px;
            font-weight: bold;
            font-size: 0.9rem;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        .method.get { background: #28a745; color: white; }
        .method.post { background: #007bff; color: white; }
        .method.put { background: #ffc107; color: black; }
        .method.delete { background: #dc3545; color: white; }
        
        .path {
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-weight: bold;
            color: #495057;
            font-size: 1.1rem;
        }
        
        .description {
            color: #6c757d;
            margin-bottom: 0.5rem;
            font-size: 1rem;
        }
        
        .category {
            display: inline-block;
            background: #e9ecef;
            color: #495057;
            padding: 0.3rem 0.8rem;
            border-radius: 20px;
            font-size: 0.8rem;
            font-weight: 500;
        }
        
        .auth-badge {
            display: inline-block;
            background: #dc3545;
            color: white;
            padding: 0.2rem 0.6rem;
            border-radius: 15px;
            font-size: 0.7rem;
            font-weight: bold;
            margin-left: 0.5rem;
        }
        
        .auth-badge.no-auth {
            background: #28a745;
        }
        
        .features {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
            margin-top: 2rem;
        }
        
        .feature {
            background: linear-gradient(135deg, #f8f9fa, #e9ecef);
            padding: 2rem;
            border-radius: 15px;
            text-align: center;
            transition: all 0.3s ease;
            border: 2px solid transparent;
        }
        
        .feature:hover {
            transform: translateY(-5px);
            border-color: #667eea;
            box-shadow: 0 15px 35px rgba(0, 0, 0, 0.1);
        }
        
        .feature h3 {
            color: #667eea;
            margin-bottom: 1rem;
            font-size: 1.3rem;
        }
        
        .feature-icon {
            font-size: 3rem;
            margin-bottom: 1rem;
            display: block;
        }
        
        .code-block {
            background: #2d3748;
            color: #e2e8f0;
            padding: 1.5rem;
            border-radius: 10px;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            margin: 1rem 0;
            overflow-x: auto;
        }
        
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin: 2rem 0;
        }
        
        .stat {
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            padding: 2rem;
            border-radius: 15px;
            text-align: center;
        }
        
        .stat-number {
            font-size: 2.5rem;
            font-weight: bold;
            margin-bottom: 0.5rem;
        }
        
        .stat-label {
            font-size: 1rem;
            opacity: 0.9;
        }
        
        @media (max-width: 768px) {
            .header h1 {
                font-size: 2rem;
            }
            
            .nav {
                flex-direction: column;
                align-items: center;
            }
            
            .endpoint-header {
                flex-direction: column;
                align-items: flex-start;
            }
        }
        
        .search-box {
            width: 100%;
            padding: 1rem;
            border: 2px solid #e9ecef;
            border-radius: 50px;
            font-size: 1rem;
            margin-bottom: 2rem;
            transition: all 0.3s ease;
        }
        
        .search-box:focus {
            outline: none;
            border-color: #667eea;
            box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 FundHub API</h1>
            <p>Comprehensive crowdfunding platform for students with Stellar blockchain integration</p>
            
            <div class="stats">
                <div class="stat">
                    <div class="stat-number">35+</div>
                    <div class="stat-label">API Endpoints</div>
                </div>
                <div class="stat">
                    <div class="stat-number">6</div>
                    <div class="stat-label">Main Categories</div>
                </div>
                <div class="stat">
                    <div class="stat-number">100%</div>
                    <div class="stat-label">RESTful</div>
                </div>
            </div>
        </div>

        <div class="nav">
            <a href="/api/docs/api" target="_blank">📊 API Information</a>
            <a href="/api/docs/health" target="_blank">❤️ Health Check</a>
            <a href="/health" target="_blank">🔍 System Health</a>
            <a href="mailto:support@fundhub.com">📧 Contact Support</a>
        </div>

        <div class="section">
            <h2>🎯 API Overview</h2>
            <p>The FundHub API provides a complete solution for student crowdfunding with the following key features:</p>
            
            <div class="features">
                <div class="feature">
                    <span class="feature-icon">👥</span>
                    <h3>Student Management</h3>
                    <p>Complete student profile management with verification system and progress tracking</p>
                </div>
                <div class="feature">
                    <span class="feature-icon">💰</span>
                    <h3>Wallet Integration</h3>
                    <p>Seamless Stellar blockchain wallet integration for secure transactions</p>
                </div>
                <div class="feature">
                    <span class="feature-icon">🎁</span>
                    <h3>Donation Processing</h3>
                    <p>Secure donation processing with real-time verification and tracking</p>
                </div>
                <div class="feature">
                    <span class="feature-icon">📊</span>
                    <h3>Campaign Management</h3>
                    <p>Advanced campaign creation and management with automated fund distribution</p>
                </div>
                <div class="feature">
                    <span class="feature-icon">📈</span>
                    <h3>Analytics & Reporting</h3>
                    <p>Comprehensive analytics and reporting for projects, students, and campaigns</p>
                </div>
                <div class="feature">
                    <span class="feature-icon">🔔</span>
                    <h3>Real-time Notifications</h3>
                    <p>Server-sent events for real-time updates and notifications</p>
                </div>
            </div>
        </div>

        <div class="section">
            <h2>🔐 Authentication</h2>
            <p>Most endpoints require authentication using Bearer tokens. Include your token in the Authorization header:</p>
            <div class="code-block">
Authorization: Bearer your_jwt_token_here
            </div>
            <p><strong>Getting Started:</strong></p>
            <ol>
                <li>Register a new account: <code>POST /api/auth/signup</code></li>
                <li>Login to get your token: <code>POST /api/auth/login</code></li>
                <li>Use the token in subsequent requests</li>
            </ol>
        </div>

        <div class="section">
            <h2>📋 Quick Start Guide</h2>
            <ol>
                <li><strong>Authentication:</strong> Register a new account or login to get your API token</li>
                <li><strong>Student Profile:</strong> Create and verify your student profile</li>
                <li><strong>Wallet Setup:</strong> Connect your Stellar wallet for transactions</li>
                <li><strong>Create Projects:</strong> Start creating and managing your crowdfunding projects</li>
                <li><strong>Campaigns:</strong> Set up campaigns to distribute funds to eligible students</li>
                <li><strong>Analytics:</strong> Monitor performance with comprehensive analytics</li>
            </ol>
        </div>

        <div class="section">
            <h2>📊 Rate Limiting</h2>
            <p>API requests are rate limited to ensure fair usage:</p>
            <ul>
                <li><strong>General endpoints:</strong> 100 requests per minute</li>
                <li><strong>Authentication endpoints:</strong> 10 requests per minute</li>
                <li><strong>Analytics endpoints:</strong> 50 requests per minute</li>
                <li><strong>Admin endpoints:</strong> 20 requests per minute</li>
            </ul>
        </div>

        <div class="section">
            <h2>🛠️ SDKs & Libraries</h2>
            <p>Official SDKs are available for popular programming languages:</p>
            <ul>
                <li><strong>JavaScript/TypeScript:</strong> <code>npm install @fundhub/api-client</code></li>
                <li><strong>Python:</strong> <code>pip install fundhub-api</code></li>
                <li><strong>Rust:</strong> <code>cargo add fundhub-api</code></li>
                <li><strong>Go:</strong> <code>go get github.com/fundhub/api-client</code></li>
            </ul>
        </div>

        <div class="section">
            <h2>🔗 Base URL</h2>
            <div class="code-block">
Development: http://localhost:3000
Production: https://api.fundhub.com
            </div>
        </div>

        <div class="section">
            <h2>📞 Support</h2>
            <p>Need help? We're here to assist you:</p>
            <ul>
                <li><strong>Email:</strong> support@fundhub.com</li>
                <li><strong>Documentation:</strong> <a href="/api/docs">Interactive API Docs</a></li>
                <li><strong>GitHub:</strong> <a href="https://github.com/fundhub/api">Report Issues</a></li>
                <li><strong>Status Page:</strong> <a href="/api/docs/health">System Status</a></li>
            </ul>
        </div>
    </div>

    <script>
        // Add some interactivity
        document.addEventListener('DOMContentLoaded', function() {
            // Add click handlers for better UX
            const navLinks = document.querySelectorAll('.nav a');
            navLinks.forEach(link => {
                link.addEventListener('click', function(e) {
                    if (this.getAttribute('href').startsWith('/')) {
                        e.preventDefault();
                        window.open(this.getAttribute('href'), '_blank');
                    }
                });
            });

            // Add search functionality
            const searchBox = document.createElement('input');
            searchBox.type = 'text';
            searchBox.placeholder = 'Search endpoints...';
            searchBox.className = 'search-box';
            
            const firstSection = document.querySelector('.section');
            firstSection.parentNode.insertBefore(searchBox, firstSection.nextSibling);
            
            // Simple search functionality
            searchBox.addEventListener('input', function() {
                const searchTerm = this.value.toLowerCase();
                const endpoints = document.querySelectorAll('.endpoint');
                
                endpoints.forEach(endpoint => {
                    const text = endpoint.textContent.toLowerCase();
                    if (text.includes(searchTerm)) {
                        endpoint.style.display = 'block';
                    } else {
                        endpoint.style.display = 'none';
                    }
                });
            });
        });
    </script>
</body>
</html>
    "#)
}