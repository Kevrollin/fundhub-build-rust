# FundHub Frontend Integration Design Guide

## 🎯 **OVERVIEW**

This comprehensive guide provides everything the frontend team needs to build a complete, polished user interface for the FundHub crowdfunding platform. The backend is fully functional with 47 API endpoints, real-time notifications, and comprehensive analytics.

## 🚀 **BACKEND CAPABILITIES SUMMARY**

### **Complete Backend Systems (12 Core Features)**
- ✅ **Authentication System** (6 endpoints) - JWT auth, email verification, role management
- ✅ **Student Verification** (5 endpoints) - School email verification, admin approval workflow
- ✅ **Stellar Blockchain Integration** (3 endpoints) - Wallet management, transaction verification
- ✅ **Project Management** (8 endpoints) - CRUD operations, status management, public access
- ✅ **Donation Processing** (4 endpoints) - Stellar transactions, automatic verification
- ✅ **Campaign Management** (8 endpoints) - Automated distribution, criteria-based targeting
- ✅ **Admin Dashboard** (8 endpoints) - Full platform administration and oversight
- ✅ **Analytics Engine** (7 endpoints) - Real-time metrics, comprehensive reporting
- ✅ **Guest System** (3 endpoints) - Anonymous donations, public project access
- ✅ **Milestone Management** (3 endpoints) - Project milestone tracking and release
- ✅ **Real-time Notifications** (1 endpoint) - Server-Sent Events for live updates
- ✅ **Background Workers** (4 active) - Automated processing and analytics

### **Technical Specifications**
- **Total API Endpoints**: 47
- **Database Tables**: 12 core tables
- **Background Workers**: 4 active workers
- **Real-time Features**: Server-Sent Events
- **Blockchain Integration**: Stellar Network (XLM/USDC)
- **Authentication**: JWT with refresh tokens
- **Analytics**: Real-time, daily, and weekly aggregation

## 🎨 **FRONTEND ARCHITECTURE DESIGN**

### **Recommended Technology Stack**
```typescript
// Core Framework
- React 18+ with TypeScript
- Next.js 14+ (App Router)
- Tailwind CSS for styling
- Framer Motion for animations

// State Management
- Zustand for global state
- React Query for server state
- React Hook Form for forms

// UI Components
- Headless UI for accessible components
- Radix UI for complex components
- Lucide React for icons

// Blockchain Integration
- Stellar SDK for wallet connections
- Web3Modal for wallet management

// Real-time Features
- EventSource for Server-Sent Events
- Socket.io client for additional real-time features
```

### **Project Structure**
```
src/
├── components/
│   ├── ui/                 # Reusable UI components
│   ├── forms/              # Form components
│   ├── charts/             # Analytics charts
│   └── layout/             # Layout components
├── pages/
│   ├── auth/               # Authentication pages
│   ├── dashboard/          # User dashboards
│   ├── projects/           # Project management
│   ├── admin/              # Admin interface
│   └── public/             # Public pages
├── hooks/
│   ├── useAuth.ts          # Authentication hooks
│   ├── useProjects.ts      # Project management hooks
│   ├── useDonations.ts     # Donation hooks
│   └── useAnalytics.ts     # Analytics hooks
├── services/
│   ├── api.ts              # API client
│   ├── stellar.ts          # Stellar integration
│   └── notifications.ts    # Real-time notifications
├── stores/
│   ├── authStore.ts        # Authentication state
│   ├── projectStore.ts     # Project state
│   └── analyticsStore.ts   # Analytics state
└── types/
    ├── api.ts              # API types
    ├── stellar.ts          # Stellar types
    └── user.ts             # User types
```

## 🔐 **AUTHENTICATION SYSTEM**

### **User Roles & Permissions**
```typescript
enum UserRole {
  GUEST = 'guest',
  USER = 'user',
  STUDENT = 'student',
  ADMIN = 'admin'
}

enum BaseRole {
  GUEST = 'guest',
  BASE_USER = 'base_user',
  STUDENT = 'student',
  ADMIN = 'admin'
}
```

### **Authentication Flow**
```typescript
// 1. User Registration
POST /api/auth/signup
{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "secure_password"
}

// 2. Email Verification
GET /api/auth/verify-email?token=verification_token

// 3. Login
POST /api/auth/login
{
  "email": "john@example.com",
  "password": "secure_password"
}

// 4. Token Refresh
POST /api/auth/refresh
{
  "refresh_token": "refresh_token_here"
}
```

### **Frontend Implementation**
```typescript
// Authentication Hook
export const useAuth = () => {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  const login = async (email: string, password: string) => {
    const response = await api.post('/auth/login', { email, password });
    const { access_token, refresh_token, user } = response.data;
    
    localStorage.setItem('access_token', access_token);
    localStorage.setItem('refresh_token', refresh_token);
    setUser(user);
  };

  const logout = async () => {
    await api.post('/auth/logout');
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    setUser(null);
  };

  return { user, login, logout, loading };
};
```

## 👨‍🎓 **STUDENT VERIFICATION SYSTEM**

### **Verification Workflow**
```typescript
// 1. Student Registration
POST /api/students/register
{
  "school_email": "john@university.edu"
}

// 2. Apply for Verification
POST /api/students/apply-verification
{
  "school_email": "john@university.edu"
}

// 3. Check Verification Status
GET /api/students/verification-status/:user_id
```

### **Frontend Implementation**
```typescript
// Student Verification Component
export const StudentVerification = () => {
  const [verificationStatus, setVerificationStatus] = useState('pending');
  const [progress, setProgress] = useState(0);

  const applyForVerification = async (schoolEmail: string) => {
    try {
      await api.post('/students/apply-verification', { school_email: schoolEmail });
      setVerificationStatus('pending');
      setProgress(25);
    } catch (error) {
      console.error('Verification application failed:', error);
    }
  };

  return (
    <div className="verification-container">
      <div className="progress-bar">
        <div className="progress" style={{ width: `${progress}%` }} />
      </div>
      <div className="status-indicator">
        Status: {verificationStatus}
      </div>
    </div>
  );
};
```

## 💰 **STELLAR BLOCKCHAIN INTEGRATION**

### **Wallet Management**
```typescript
// 1. Connect Wallet
POST /api/wallets/connect
{
  "public_key": "stellar_public_key_here"
}

// 2. Get Balance
GET /api/wallets/balance/:wallet_id

// 3. Get Transactions
GET /api/wallets/transactions/:wallet_id
```

### **Frontend Implementation**
```typescript
// Stellar Wallet Hook
export const useStellarWallet = () => {
  const [wallet, setWallet] = useState<Wallet | null>(null);
  const [balance, setBalance] = useState<WalletBalance | null>(null);

  const connectWallet = async (publicKey: string) => {
    try {
      const response = await api.post('/wallets/connect', { public_key: publicKey });
      setWallet(response.data);
    } catch (error) {
      console.error('Wallet connection failed:', error);
    }
  };

  const fetchBalance = async (walletId: string) => {
    try {
      const response = await api.get(`/wallets/balance/${walletId}`);
      setBalance(response.data);
    } catch (error) {
      console.error('Balance fetch failed:', error);
    }
  };

  return { wallet, balance, connectWallet, fetchBalance };
};
```

## 📁 **PROJECT MANAGEMENT SYSTEM**

### **Project Operations**
```typescript
// 1. Create Project
POST /api/projects
{
  "title": "AI Research Project",
  "description": "Machine learning research for healthcare",
  "repo_url": "https://github.com/user/project",
  "media_url": "https://example.com/video.mp4",
  "tags": ["AI", "Healthcare", "Research"],
  "funding_goal": 1000.00
}

// 2. Get Projects
GET /api/projects
GET /api/projects/public

// 3. Update Project
PUT /api/projects/:id

// 4. Publish Project
POST /api/projects/:id/publish
```

### **Frontend Implementation**
```typescript
// Project Management Hook
export const useProjects = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(false);

  const createProject = async (projectData: CreateProjectRequest) => {
    try {
      const response = await api.post('/projects', projectData);
      setProjects(prev => [...prev, response.data]);
      return response.data;
    } catch (error) {
      console.error('Project creation failed:', error);
      throw error;
    }
  };

  const fetchProjects = async () => {
    setLoading(true);
    try {
      const response = await api.get('/projects');
      setProjects(response.data);
    } catch (error) {
      console.error('Projects fetch failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return { projects, createProject, fetchProjects, loading };
};
```

## 💸 **DONATION PROCESSING SYSTEM**

### **Donation Flow**
```typescript
// 1. Initiate Donation
POST /api/donations/initiate
{
  "project_id": "project_uuid",
  "amount": 50.00,
  "payment_method": "stellar"
}

// 2. Verify Donation
POST /api/donations/verify
{
  "tx_hash": "stellar_transaction_hash",
  "project_id": "project_uuid"
}

// 3. Get Project Donations
GET /api/donations/project/:project_id
```

### **Frontend Implementation**
```typescript
// Donation Component
export const DonationForm = ({ projectId }: { projectId: string }) => {
  const [amount, setAmount] = useState(0);
  const [txHash, setTxHash] = useState('');

  const initiateDonation = async () => {
    try {
      const response = await api.post('/donations/initiate', {
        project_id: projectId,
        amount: amount,
        payment_method: 'stellar'
      });
      return response.data;
    } catch (error) {
      console.error('Donation initiation failed:', error);
    }
  };

  const verifyDonation = async () => {
    try {
      const response = await api.post('/donations/verify', {
        tx_hash: txHash,
        project_id: projectId
      });
      return response.data;
    } catch (error) {
      console.error('Donation verification failed:', error);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="number"
        value={amount}
        onChange={(e) => setAmount(Number(e.target.value))}
        placeholder="Donation amount"
      />
      <input
        type="text"
        value={txHash}
        onChange={(e) => setTxHash(e.target.value)}
        placeholder="Transaction hash"
      />
      <button type="submit">Donate</button>
    </form>
  );
};
```

## 🎯 **CAMPAIGN MANAGEMENT SYSTEM**

### **Campaign Operations**
```typescript
// 1. Create Campaign
POST /api/campaigns/create
{
  "name": "Student Reward Campaign",
  "description": "Reward all verified students",
  "criteria": "verified_students",
  "reward_pool_xlm": 1000.00
}

// 2. Execute Campaign
POST /api/campaigns/execute
{
  "campaign_id": "campaign_uuid"
}

// 3. Get Campaign Stats
GET /api/campaigns/stats
```

### **Frontend Implementation**
```typescript
// Campaign Management Hook
export const useCampaigns = () => {
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [stats, setStats] = useState<CampaignStats | null>(null);

  const createCampaign = async (campaignData: CreateCampaignRequest) => {
    try {
      const response = await api.post('/campaigns/create', campaignData);
      setCampaigns(prev => [...prev, response.data]);
      return response.data;
    } catch (error) {
      console.error('Campaign creation failed:', error);
    }
  };

  const executeCampaign = async (campaignId: string) => {
    try {
      const response = await api.post('/campaigns/execute', { campaign_id: campaignId });
      return response.data;
    } catch (error) {
      console.error('Campaign execution failed:', error);
    }
  };

  return { campaigns, stats, createCampaign, executeCampaign };
};
```

## 📊 **ANALYTICS DASHBOARD**

### **Analytics Endpoints**
```typescript
// 1. Platform Stats
GET /api/analytics/platform/stats

// 2. Top Projects
GET /api/analytics/projects/top

// 3. Top Students
GET /api/analytics/students/top

// 4. Campaign Performance
GET /api/analytics/campaigns/performance

// 5. Donation Trends
GET /api/analytics/donations/trends

// 6. Project Analytics
GET /api/analytics/projects/:id

// 7. Student Analytics
GET /api/analytics/students/:id
```

### **Frontend Implementation**
```typescript
// Analytics Dashboard Component
export const AnalyticsDashboard = () => {
  const [platformStats, setPlatformStats] = useState<PlatformStats | null>(null);
  const [topProjects, setTopProjects] = useState<Project[]>([]);
  const [donationTrends, setDonationTrends] = useState<DonationTrend[]>([]);

  useEffect(() => {
    const fetchAnalytics = async () => {
      try {
        const [statsRes, projectsRes, trendsRes] = await Promise.all([
          api.get('/analytics/platform/stats'),
          api.get('/analytics/projects/top'),
          api.get('/analytics/donations/trends')
        ]);

        setPlatformStats(statsRes.data);
        setTopProjects(projectsRes.data);
        setDonationTrends(trendsRes.data);
      } catch (error) {
        console.error('Analytics fetch failed:', error);
      }
    };

    fetchAnalytics();
  }, []);

  return (
    <div className="analytics-dashboard">
      <div className="stats-grid">
        <StatCard title="Total Users" value={platformStats?.total_users} />
        <StatCard title="Active Projects" value={platformStats?.active_projects} />
        <StatCard title="Total Donations" value={platformStats?.total_donations} />
      </div>
      <div className="charts">
        <DonationTrendChart data={donationTrends} />
        <TopProjectsChart data={topProjects} />
      </div>
    </div>
  );
};
```

## 🔔 **REAL-TIME NOTIFICATIONS**

### **Server-Sent Events Integration**
```typescript
// Real-time Notifications Hook
export const useNotifications = () => {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const eventSource = new EventSource('/api/notifications/stream');

    eventSource.onopen = () => {
      setIsConnected(true);
    };

    eventSource.onmessage = (event) => {
      const notification = JSON.parse(event.data);
      setNotifications(prev => [notification, ...prev]);
    };

    eventSource.onerror = () => {
      setIsConnected(false);
    };

    return () => {
      eventSource.close();
    };
  }, []);

  return { notifications, isConnected };
};
```

## 👤 **ADMIN DASHBOARD**

### **Admin Operations**
```typescript
// 1. List Students
GET /api/admin/students

// 2. List Pending Verifications
GET /api/admin/verifications

// 3. Approve/Reject Verification
POST /api/admin/verifications/:id/approve
POST /api/admin/verifications/:id/reject

// 4. Fund Student Directly
POST /api/admin/fund-student
{
  "student_id": "student_uuid",
  "amount": 100.00
}

// 5. Get Activity Logs
GET /api/admin/logs

// 6. Get Admin Overview
GET /api/admin/overview
```

### **Frontend Implementation**
```typescript
// Admin Dashboard Component
export const AdminDashboard = () => {
  const [students, setStudents] = useState<Student[]>([]);
  const [pendingVerifications, setPendingVerifications] = useState<Verification[]>([]);
  const [activityLogs, setActivityLogs] = useState<ActivityLog[]>([]);

  const approveVerification = async (verificationId: string) => {
    try {
      await api.post(`/admin/verifications/${verificationId}/approve`);
      // Refresh data
      fetchPendingVerifications();
    } catch (error) {
      console.error('Verification approval failed:', error);
    }
  };

  const rejectVerification = async (verificationId: string, reason: string) => {
    try {
      await api.post(`/admin/verifications/${verificationId}/reject`, { reason });
      // Refresh data
      fetchPendingVerifications();
    } catch (error) {
      console.error('Verification rejection failed:', error);
    }
  };

  return (
    <div className="admin-dashboard">
      <div className="verification-queue">
        <h2>Pending Verifications</h2>
        {pendingVerifications.map(verification => (
          <VerificationCard
            key={verification.id}
            verification={verification}
            onApprove={() => approveVerification(verification.id)}
            onReject={(reason) => rejectVerification(verification.id, reason)}
          />
        ))}
      </div>
    </div>
  );
};
```

## 🎯 **USER LIFECYCLE & USE CASES**

### **1. Guest User Journey**
```typescript
// Guest can:
// - View public projects
// - Make donations without registration
// - Access project information

const GuestUserFlow = () => {
  return (
    <div className="guest-flow">
      <PublicProjectsList />
      <GuestDonationForm />
    </div>
  );
};
```

### **2. Registered User Journey**
```typescript
// Registered user can:
// - Create account and verify email
// - Apply for student verification
// - View projects and make donations
// - Access basic analytics

const RegisteredUserFlow = () => {
  return (
    <div className="user-flow">
      <UserDashboard />
      <ProjectBrowser />
      <DonationHistory />
    </div>
  );
};
```

### **3. Verified Student Journey**
```typescript
// Verified student can:
// - Connect Stellar wallet
// - Create and manage projects
// - Set up milestones
// - Receive donations
// - Access student analytics

const StudentFlow = () => {
  return (
    <div className="student-flow">
      <StudentDashboard />
      <ProjectManagement />
      <WalletManagement />
      <MilestoneTracking />
    </div>
  );
};
```

### **4. Admin Journey**
```typescript
// Admin can:
// - Manage student verifications
// - Create and execute campaigns
// - Access comprehensive analytics
// - Manage platform settings
// - View activity logs

const AdminFlow = () => {
  return (
    <div className="admin-flow">
      <AdminDashboard />
      <VerificationManagement />
      <CampaignManagement />
      <AnalyticsDashboard />
    </div>
  );
};
```

## 🎨 **UI/UX DESIGN GUIDELINES**

### **Design System**
```typescript
// Color Palette
const colors = {
  primary: '#3B82F6',      // Blue
  secondary: '#10B981',    // Green
  accent: '#F59E0B',       // Amber
  danger: '#EF4444',       // Red
  success: '#10B981',      // Green
  warning: '#F59E0B',      // Amber
  info: '#3B82F6',         // Blue
  dark: '#1F2937',         // Dark gray
  light: '#F9FAFB'         // Light gray
};

// Typography
const typography = {
  fontFamily: 'Inter, sans-serif',
  sizes: {
    xs: '0.75rem',
    sm: '0.875rem',
    base: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
    '2xl': '1.5rem',
    '3xl': '1.875rem',
    '4xl': '2.25rem'
  }
};

// Spacing
const spacing = {
  xs: '0.25rem',
  sm: '0.5rem',
  md: '1rem',
  lg: '1.5rem',
  xl: '2rem',
  '2xl': '3rem'
};
```

### **Component Library**
```typescript
// Button Component
export const Button = ({ variant, size, children, ...props }) => {
  const baseClasses = 'font-medium rounded-lg transition-colors';
  const variants = {
    primary: 'bg-blue-600 text-white hover:bg-blue-700',
    secondary: 'bg-gray-200 text-gray-900 hover:bg-gray-300',
    danger: 'bg-red-600 text-white hover:bg-red-700'
  };
  const sizes = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-base',
    lg: 'px-6 py-3 text-lg'
  };

  return (
    <button
      className={`${baseClasses} ${variants[variant]} ${sizes[size]}`}
      {...props}
    >
      {children}
    </button>
  );
};

// Card Component
export const Card = ({ children, className, ...props }) => {
  return (
    <div
      className={`bg-white rounded-lg shadow-md border border-gray-200 ${className}`}
      {...props}
    >
      {children}
    </div>
  );
};
```

## 📱 **RESPONSIVE DESIGN**

### **Breakpoints**
```typescript
const breakpoints = {
  sm: '640px',
  md: '768px',
  lg: '1024px',
  xl: '1280px',
  '2xl': '1536px'
};

// Mobile-first approach
const responsiveClasses = {
  container: 'w-full px-4 sm:px-6 lg:px-8',
  grid: 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6',
  sidebar: 'hidden lg:block lg:w-64',
  main: 'lg:ml-64'
};
```

### **Mobile Navigation**
```typescript
export const MobileNavigation = () => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="lg:hidden">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="p-2 rounded-md text-gray-600 hover:text-gray-900"
      >
        <MenuIcon className="h-6 w-6" />
      </button>
      
      {isOpen && (
        <div className="absolute top-16 left-0 right-0 bg-white shadow-lg">
          <NavigationMenu />
        </div>
      )}
    </div>
  );
};
```

## 🔒 **SECURITY IMPLEMENTATION**

### **Authentication Guards**
```typescript
// Route Protection
export const ProtectedRoute = ({ children, requiredRole }) => {
  const { user, loading } = useAuth();

  if (loading) return <LoadingSpinner />;
  if (!user) return <Navigate to="/login" />;
  if (requiredRole && user.role !== requiredRole) {
    return <Navigate to="/unauthorized" />;
  }

  return children;
};

// API Client with Auth
const apiClient = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL,
});

apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      // Try to refresh token
      const refreshToken = localStorage.getItem('refresh_token');
      if (refreshToken) {
        try {
          const response = await apiClient.post('/auth/refresh', {
            refresh_token: refreshToken
          });
          const { access_token } = response.data;
          localStorage.setItem('access_token', access_token);
          return apiClient.request(error.config);
        } catch (refreshError) {
          // Redirect to login
          localStorage.removeItem('access_token');
          localStorage.removeItem('refresh_token');
          window.location.href = '/login';
        }
      }
    }
    return Promise.reject(error);
  }
);
```

## 🧪 **TESTING STRATEGY**

### **Unit Tests**
```typescript
// Component Testing
import { render, screen, fireEvent } from '@testing-library/react';
import { DonationForm } from './DonationForm';

describe('DonationForm', () => {
  it('should submit donation with valid data', async () => {
    const mockSubmit = jest.fn();
    render(<DonationForm onSubmit={mockSubmit} />);
    
    fireEvent.change(screen.getByLabelText('Amount'), { target: { value: '50' } });
    fireEvent.change(screen.getByLabelText('Transaction Hash'), { target: { value: 'test_hash' } });
    fireEvent.click(screen.getByText('Donate'));
    
    expect(mockSubmit).toHaveBeenCalledWith({
      amount: 50,
      txHash: 'test_hash'
    });
  });
});
```

### **Integration Tests**
```typescript
// API Integration Testing
import { api } from './services/api';

describe('API Integration', () => {
  it('should fetch projects successfully', async () => {
    const response = await api.get('/projects');
    expect(response.status).toBe(200);
    expect(response.data).toBeInstanceOf(Array);
  });
});
```

## 🚀 **DEPLOYMENT CONFIGURATION**

### **Environment Variables**
```bash
# Frontend Environment Variables
NEXT_PUBLIC_API_URL=http://localhost:3000/api
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_HORIZON_URL=https://horizon-testnet.stellar.org
NEXT_PUBLIC_APP_NAME=FundHub
NEXT_PUBLIC_APP_VERSION=1.0.0
```

### **Build Configuration**
```typescript
// next.config.js
const nextConfig = {
  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL,
    NEXT_PUBLIC_STELLAR_NETWORK: process.env.NEXT_PUBLIC_STELLAR_NETWORK,
  },
  images: {
    domains: ['example.com', 'github.com'],
  },
  experimental: {
    appDir: true,
  },
};

module.exports = nextConfig;
```

## 📊 **PERFORMANCE OPTIMIZATION**

### **Code Splitting**
```typescript
// Lazy Loading Components
const AdminDashboard = lazy(() => import('./pages/AdminDashboard'));
const AnalyticsDashboard = lazy(() => import('./pages/AnalyticsDashboard'));

// Route-based Code Splitting
const routes = [
  {
    path: '/admin',
    component: lazy(() => import('./pages/AdminDashboard')),
    exact: true
  },
  {
    path: '/analytics',
    component: lazy(() => import('./pages/AnalyticsDashboard')),
    exact: true
  }
];
```

### **Caching Strategy**
```typescript
// React Query Configuration
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
      retry: 3,
      retryDelay: attemptIndex => Math.min(1000 * 2 ** attemptIndex, 30000),
    },
  },
});
```

## 🎯 **COMPLETE IMPLEMENTATION CHECKLIST**

### **Phase 1: Core Authentication & User Management**
- [ ] User registration and login forms
- [ ] Email verification system
- [ ] Password reset functionality
- [ ] Role-based access control
- [ ] Student verification workflow
- [ ] Admin verification management

### **Phase 2: Project Management**
- [ ] Project creation and editing forms
- [ ] Project listing and search
- [ ] Project detail pages
- [ ] Media upload and management
- [ ] Tag system implementation
- [ ] Project status management

### **Phase 3: Blockchain Integration**
- [ ] Stellar wallet connection
- [ ] Wallet balance display
- [ ] Transaction history
- [ ] Donation processing
- [ ] Transaction verification
- [ ] Multi-asset support

### **Phase 4: Donation System**
- [ ] Donation forms
- [ ] Payment method selection
- [ ] Transaction verification
- [ ] Donation history
- [ ] Receipt generation
- [ ] Guest donation support

### **Phase 5: Analytics Dashboard**
- [ ] Platform statistics
- [ ] Project performance metrics
- [ ] Student success tracking
- [ ] Campaign effectiveness
- [ ] Donation trends
- [ ] Real-time updates

### **Phase 6: Admin Features**
- [ ] Admin dashboard
- [ ] Student verification management
- [ ] Campaign creation and execution
- [ ] Activity logging
- [ ] Platform oversight
- [ ] Direct funding capabilities

### **Phase 7: Real-time Features**
- [ ] Server-Sent Events integration
- [ ] Live notifications
- [ ] Real-time updates
- [ ] Connection management
- [ ] Notification preferences

### **Phase 8: Mobile Optimization**
- [ ] Responsive design
- [ ] Mobile navigation
- [ ] Touch-friendly interfaces
- [ ] Mobile-specific features
- [ ] Performance optimization

## 🎨 **DESIGN INSPIRATION & REFERENCES**

### **Color Schemes**
- **Primary**: Modern blue (#3B82F6) for trust and professionalism
- **Secondary**: Green (#10B981) for success and growth
- **Accent**: Amber (#F59E0B) for attention and highlights
- **Neutral**: Gray scale for text and backgrounds

### **Typography**
- **Font Family**: Inter for modern, readable text
- **Headings**: Bold weights for hierarchy
- **Body**: Regular weights for readability
- **Code**: Monospace for technical content

### **Component Design**
- **Cards**: Subtle shadows and rounded corners
- **Buttons**: Clear hierarchy with hover states
- **Forms**: Clean inputs with validation states
- **Navigation**: Intuitive menu structure
- **Charts**: Modern data visualization

## 🚀 **GETTING STARTED**

### **Quick Start Commands**
```bash
# 1. Create Next.js project
npx create-next-app@latest fundhub-frontend --typescript --tailwind --app

# 2. Install dependencies
npm install @tanstack/react-query zustand framer-motion
npm install @headlessui/react @radix-ui/react-dialog
npm install lucide-react axios

# 3. Set up environment variables
cp .env.example .env.local

# 4. Start development server
npm run dev
```

### **Project Structure Setup**
```bash
# Create directory structure
mkdir -p src/{components,pages,hooks,services,stores,types}
mkdir -p src/components/{ui,forms,charts,layout}
mkdir -p src/pages/{auth,dashboard,projects,admin,public}
```

## 📞 **SUPPORT & RESOURCES**

### **Backend API Documentation**
- **Base URL**: `http://localhost:3000/api`
- **Swagger UI**: `http://localhost:3000/api/docs`
- **Health Check**: `http://localhost:3000/health`

### **Key Endpoints Summary**
- **Authentication**: 6 endpoints
- **Student Management**: 5 endpoints
- **Project Management**: 8 endpoints
- **Donation Processing**: 4 endpoints
- **Campaign Management**: 8 endpoints
- **Admin Dashboard**: 8 endpoints
- **Analytics**: 7 endpoints
- **Real-time**: 1 endpoint

### **Development Tips**
1. **Start with authentication** - Get user management working first
2. **Implement project management** - Core functionality for students
3. **Add blockchain integration** - Stellar wallet connection
4. **Build donation system** - Payment processing
5. **Create analytics dashboard** - Data visualization
6. **Implement admin features** - Platform management
7. **Add real-time features** - Live updates
8. **Optimize for mobile** - Responsive design

This comprehensive guide provides everything needed to build a complete, polished frontend for the FundHub platform. The backend is fully functional with 47 API endpoints, real-time notifications, and comprehensive analytics capabilities.
