# FundHub Production Deployment Checklist

Use this checklist to ensure a secure and reliable production deployment.

## Pre-Deployment

### Security
- [ ] Change all default passwords and secrets in `.env`
- [ ] Generate strong JWT secret (use `openssl rand -base64 32`)
- [ ] Set up KMS for wallet private keys (AWS KMS, Google Cloud KMS, or HashiCorp Vault)
- [ ] Remove `PLATFORM_WALLET_PRIVATE_KEY` from environment (use KMS instead)
- [ ] Configure CORS with specific allowed origins
- [ ] Enable HTTPS/TLS certificates (Let's Encrypt, AWS ACM, etc.)
- [ ] Set up API rate limiting
- [ ] Enable request validation middleware
- [ ] Configure secure session management
- [ ] Set up WAF (Web Application Firewall) if available

### Database
- [ ] Set up production PostgreSQL instance (RDS, Cloud SQL, etc.)
- [ ] Enable SSL connections to database
- [ ] Configure automated backups (daily at minimum)
- [ ] Set up point-in-time recovery
- [ ] Configure connection pooling limits
- [ ] Set up read replicas if needed
- [ ] Test backup restoration procedure
- [ ] Document database credentials management

### Infrastructure
- [ ] Provision production servers/containers
- [ ] Set up load balancer (ALB, nginx, etc.)
- [ ] Configure auto-scaling if using containers
- [ ] Set up CDN for static assets (CloudFront, Cloudflare, etc.)
- [ ] Configure DNS records
- [ ] Set up health check endpoints monitoring
- [ ] Configure log aggregation (CloudWatch, ELK, etc.)
- [ ] Set up metrics collection (Prometheus, Datadog, etc.)

### Stellar/Blockchain
- [ ] Create production Stellar accounts
- [ ] Fund platform wallet with XLM
- [ ] Deploy Soroban contracts to mainnet
- [ ] Test contract functionality on mainnet
- [ ] Document contract addresses
- [ ] Set up contract upgrade procedure
- [ ] Configure Horizon API endpoints for mainnet
- [ ] Test transaction submissions

### Storage
- [ ] Set up production S3/MinIO
- [ ] Configure bucket policies and IAM roles
- [ ] Enable versioning on buckets
- [ ] Set up lifecycle policies
- [ ] Configure CDN for file delivery
- [ ] Test file upload/download

### Monitoring & Alerting
- [ ] Set up application monitoring (New Relic, Datadog, etc.)
- [ ] Configure error tracking (Sentry, Rollbar, etc.)
- [ ] Set up uptime monitoring (Pingdom, UptimeRobot, etc.)
- [ ] Configure alerts for:
  - [ ] API response time > 2s
  - [ ] Error rate > 1%
  - [ ] Database connection failures
  - [ ] Worker failures
  - [ ] Disk usage > 80%
  - [ ] Memory usage > 85%
  - [ ] Failed Stellar transactions
- [ ] Set up on-call rotation
- [ ] Document incident response procedures

## Deployment Steps

### 1. Environment Setup
```bash
# Set production environment variables
export DATABASE_URL="postgresql://..."
export REDIS_URL="redis://..."
export JWT_SECRET="<strong-secret>"
export STELLAR_NETWORK="public"
export STELLAR_HORIZON_URL="https://horizon.stellar.org"
# ... other vars
```

### 2. Database Migration
```bash
# Backup current database
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d_%H%M%S).sql

# Run migrations
./scripts/run_migrations.sh

# Verify migrations
psql $DATABASE_URL -c "SELECT * FROM schema_migrations ORDER BY applied_at DESC LIMIT 5;"
```

### 3. Build Application
```bash
# Build release binary
cargo build --release

# Verify binary
./target/release/fundhub --version
```

### 4. Deploy Contracts
```bash
cd contracts

# Deploy to mainnet
./deploy.sh public

# Save contract addresses
# Update .env with contract addresses
```

### 5. Deploy Application
```bash
# Option A: Docker
docker build -t fundhub-api:v1.0.0 .
docker push fundhub-api:v1.0.0
docker-compose -f docker-compose.prod.yml up -d

# Option B: Kubernetes
kubectl apply -f k8s/

# Option C: Direct binary
systemctl restart fundhub-api
```

### 6. Start Workers
```bash
# Start indexer
systemctl start fundhub-indexer

# Verify workers are running
systemctl status fundhub-api
systemctl status fundhub-indexer
```

### 7. Verification
```bash
# Health check
curl https://api.fundhub.io/health

# Test authentication
curl -X POST https://api.fundhub.io/api/auth/signup -d '{...}'

# Verify database connections
psql $DATABASE_URL -c "SELECT COUNT(*) FROM users;"

# Check worker logs
journalctl -u fundhub-api -f
```

## Post-Deployment

### Immediate Checks (First 24 hours)
- [ ] Monitor error rates
- [ ] Check API response times
- [ ] Verify background workers are running
- [ ] Monitor database performance
- [ ] Check Stellar transaction success rate
- [ ] Verify SSE connections
- [ ] Monitor memory/CPU usage
- [ ] Check disk space

### Day 2-7
- [ ] Review error logs daily
- [ ] Monitor donation verification accuracy
- [ ] Check wallet sync success rate
- [ ] Review analytics worker performance
- [ ] Monitor database growth
- [ ] Check backup completion
- [ ] Review security logs

### Ongoing Maintenance
- [ ] Weekly security updates
- [ ] Monthly dependency updates
- [ ] Quarterly disaster recovery testing
- [ ] Continuous performance optimization
- [ ] Regular security audits

## Rollback Plan

### Quick Rollback
```bash
# Revert to previous version
docker pull fundhub-api:v0.9.0
docker-compose up -d

# Or with Kubernetes
kubectl rollout undo deployment/fundhub-api
```

### Database Rollback
```bash
# Restore from backup
pg_restore -d $DATABASE_URL backup_YYYYMMDD_HHMMSS.sql

# Or revert specific migration
# (requires manual SQL)
```

## Performance Tuning

### Database
- [ ] Add indexes for frequently queried fields
- [ ] Optimize slow queries (check pg_stat_statements)
- [ ] Tune PostgreSQL configuration
- [ ] Configure connection pooling (PgBouncer)
- [ ] Set up query caching

### API
- [ ] Enable response compression
- [ ] Implement request caching (Redis)
- [ ] Optimize JSON serialization
- [ ] Reduce database roundtrips
- [ ] Use connection pooling

### Workers
- [ ] Tune polling intervals
- [ ] Implement exponential backoff
- [ ] Add worker health checks
- [ ] Monitor worker lag
- [ ] Scale workers horizontally if needed

## Security Hardening

### Network
- [ ] Configure VPC/security groups
- [ ] Restrict database access to application only
- [ ] Enable DDoS protection
- [ ] Set up intrusion detection
- [ ] Configure firewall rules

### Application
- [ ] Enable helmet middleware (security headers)
- [ ] Implement input sanitization
- [ ] Add SQL injection prevention checks
- [ ] Configure CORS properly
- [ ] Enable CSRF protection
- [ ] Implement rate limiting per user
- [ ] Add request size limits

### Secrets Management
- [ ] Move secrets to vault (HashiCorp Vault, AWS Secrets Manager)
- [ ] Implement secret rotation
- [ ] Use IAM roles instead of keys where possible
- [ ] Audit secret access
- [ ] Document secret recovery procedures

## Compliance

### Data Privacy
- [ ] Implement GDPR compliance (if applicable)
- [ ] Add data retention policies
- [ ] Implement right to deletion
- [ ] Set up data export functionality
- [ ] Document data processing

### Audit Logging
- [ ] Log all administrative actions
- [ ] Log authentication events
- [ ] Log financial transactions
- [ ] Set up log retention (90+ days)
- [ ] Implement log analysis

## Documentation

### Technical
- [ ] Update API documentation
- [ ] Document deployment procedure
- [ ] Create runbook for common issues
- [ ] Document disaster recovery
- [ ] Create architecture diagrams

### Operational
- [ ] Create on-call guide
- [ ] Document escalation procedures
- [ ] Create troubleshooting guide
- [ ] Document monitoring dashboards
- [ ] Create status page

## Cost Optimization

- [ ] Right-size instances
- [ ] Set up auto-scaling
- [ ] Use spot instances for workers (if applicable)
- [ ] Implement database read replicas
- [ ] Set up S3 lifecycle policies
- [ ] Monitor and optimize query costs
- [ ] Review Stellar transaction costs

## Testing in Production

- [ ] Implement feature flags
- [ ] Set up A/B testing framework
- [ ] Create staging environment
- [ ] Implement canary deployments
- [ ] Set up blue-green deployment

## Disaster Recovery

### Backup Strategy
- [ ] Database: Daily full backups, hourly incremental
- [ ] Files: Daily backups with 30-day retention
- [ ] Configurations: Version controlled
- [ ] Secrets: Securely backed up
- [ ] Test restoration monthly

### Recovery Procedures
- [ ] Document RTO (Recovery Time Objective)
- [ ] Document RPO (Recovery Point Objective)
- [ ] Create disaster recovery runbook
- [ ] Test failover procedures quarterly
- [ ] Document emergency contacts

## Launch Checklist

### Final Checks
- [ ] All tests passing
- [ ] Security scan completed
- [ ] Performance testing done
- [ ] Load testing completed
- [ ] Backup restoration tested
- [ ] Monitoring configured
- [ ] Alerts set up
- [ ] Documentation updated
- [ ] Team trained
- [ ] Support team ready

### Communication
- [ ] Notify users of launch
- [ ] Prepare status page
- [ ] Set up support channels
- [ ] Create FAQ
- [ ] Prepare press release (if applicable)

### Go/No-Go Criteria
- [ ] All critical bugs resolved
- [ ] Performance meets SLA
- [ ] Security audit passed
- [ ] Backup/restore tested
- [ ] Team signed off
- [ ] Stakeholders approved

---

## Emergency Contacts

**DevOps Team:**
- Primary: [contact]
- Secondary: [contact]

**Database Admin:**
- Primary: [contact]

**Security Team:**
- Primary: [contact]

**On-Call:**
- Rotation: [link]
- Escalation: [procedure]

---

**Last Updated:** [Date]
**Next Review:** [Date]

