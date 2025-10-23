# Student Verification Flow - Test Guide

## Overview
This document outlines the comprehensive testing process for the enhanced student verification system in the Fundhub Platform.

## Test Scenarios

### 1. Base User Journey

#### 1.1 User Registration and Initial State
- [ ] User registers with basic account (role: user, base_role: base_user)
- [ ] User verification_status is 'not_applied'
- [ ] User can access profile but sees "Become a Verified Student" button
- [ ] User cannot access student dashboard or create projects

#### 1.2 Verification Application Process
- [ ] User clicks "Become a Verified Student" button
- [ ] Modal/form appears with enhanced fields:
  - [ ] Full name (required)
  - [ ] School/University name (required)
  - [ ] School email (required, must end with .edu or .ac.ke)
  - [ ] Student bio (optional)
  - [ ] Motivation text (optional)
- [ ] Form validation works correctly
- [ ] Email validation rejects non-.edu/.ac.ke domains
- [ ] Submission creates verification request with status 'pending'
- [ ] User verification_status updates to 'pending'
- [ ] User sees progress indicator (Step 1 of 3: Submitted)

### 2. Admin Review Process

#### 2.1 Admin Dashboard Access
- [ ] Admin can access /admin/dashboard
- [ ] Admin sees enhanced verification management interface
- [ ] Admin can view all verification requests with detailed information
- [ ] Admin can filter by status (Pending, Verified, Rejected)
- [ ] Admin can search by name, email, or school

#### 2.2 Admin Review Actions
- [ ] Admin can view detailed verification information
- [ ] Admin can approve verification with optional message
- [ ] Admin can reject verification with required reason
- [ ] Admin actions update verification status correctly
- [ ] Admin actions are logged in activity_logs
- [ ] Admin actions create verification_history entries

### 3. Student Status Updates

#### 3.1 Real-time Updates
- [ ] Student receives real-time status updates via SSE
- [ ] Status changes trigger UI updates without page refresh
- [ ] Toast notifications appear for status changes
- [ ] Student dashboard shows appropriate status alerts

#### 3.2 Verified Student Experience
- [ ] Upon approval, user role changes to 'student'
- [ ] User verification_status becomes 'verified'
- [ ] User can access student dashboard
- [ ] User can create projects
- [ ] User can connect Stellar wallet
- [ ] User can receive donations

#### 3.3 Rejected Student Experience
- [ ] Upon rejection, user verification_status becomes 'rejected'
- [ ] User sees rejection reason from admin
- [ ] User can reapply for verification
- [ ] User cannot access student features

### 4. UI/UX Testing

#### 4.1 Visual Design
- [ ] Clean, modern interface with glassmorphic cards
- [ ] Proper color coding for status badges:
  - [ ] Gray for Pending
  - [ ] Green for Verified
  - [ ] Red for Rejected
- [ ] Smooth transitions and animations
- [ ] Responsive design works on mobile and desktop

#### 4.2 User Experience
- [ ] Intuitive navigation and clear call-to-actions
- [ ] Progress indicators show current step
- [ ] Error messages are helpful and actionable
- [ ] Success messages provide clear next steps
- [ ] Loading states are smooth and informative

### 5. Database Testing

#### 5.1 Data Integrity
- [ ] All new fields are properly stored
- [ ] Foreign key relationships are maintained
- [ ] Verification history is tracked correctly
- [ ] User role updates are atomic
- [ ] Activity logs are comprehensive

#### 5.2 Performance
- [ ] Database queries are optimized
- [ ] Indexes are properly created
- [ ] Large datasets perform well
- [ ] Concurrent operations work correctly

### 6. API Testing

#### 6.1 Student Endpoints
- [ ] POST /api/students/apply-verification (enhanced)
- [ ] GET /api/students/verification-status/{user_id}
- [ ] GET /api/students/profile/{user_id}
- [ ] PUT /api/students/profile/{user_id}

#### 6.2 Admin Endpoints
- [ ] GET /api/admin/verifications/enhanced
- [ ] GET /api/admin/verifications/{id}/details
- [ ] POST /api/admin/verifications/{id}/approve-enhanced
- [ ] POST /api/admin/verifications/{id}/reject-enhanced

#### 6.3 Error Handling
- [ ] Proper HTTP status codes
- [ ] Meaningful error messages
- [ ] Input validation
- [ ] Authentication/authorization checks

### 7. Integration Testing

#### 7.1 End-to-End Flow
- [ ] Complete user journey from registration to verified student
- [ ] Admin approval workflow
- [ ] Real-time status updates
- [ ] Student dashboard access after verification

#### 7.2 Edge Cases
- [ ] Multiple verification attempts
- [ ] Invalid email formats
- [ ] Network failures during submission
- [ ] Concurrent admin actions
- [ ] User role conflicts

## Test Data

### Sample Student Application
```json
{
  "full_name": "John Doe",
  "school_name": "University of California",
  "school_email": "john.doe@uc.edu",
  "student_bio": "Computer Science student passionate about blockchain technology",
  "motivation_text": "I want to build innovative projects and connect with the developer community"
}
```

### Sample Admin Approval
```json
{
  "admin_id": "00000000-0000-0000-0000-000000000001",
  "message": "Welcome to the platform! Your verification has been approved."
}
```

### Sample Admin Rejection
```json
{
  "reason": "Please provide a valid .edu email address from your institution."
}
```

## Success Criteria

- [ ] All test scenarios pass
- [ ] No critical bugs or errors
- [ ] Performance meets requirements
- [ ] UI/UX is intuitive and professional
- [ ] Real-time updates work reliably
- [ ] Database integrity is maintained
- [ ] Security requirements are met

## Rollback Plan

If issues are discovered:
1. Revert database migrations
2. Restore previous API endpoints
3. Revert frontend components
4. Update documentation
5. Notify users of temporary unavailability

## Post-Deployment Monitoring

- [ ] Monitor verification application rates
- [ ] Track admin approval times
- [ ] Monitor error rates and performance
- [ ] Collect user feedback
- [ ] Analyze usage patterns
