# Deployment Guide

## Prerequisites

- Vercel account
- Supabase account
- Google Gemini API key
- ElevenLabs API key
- Git repository

## Step 1: Set up Supabase

1. Create a new Supabase project
2. Note your project URL and anonymous key
3. Enable pgvector extension:
   - Go to SQL Editor
   - Run: `CREATE EXTENSION IF NOT EXISTS vector;`
4. Create a database connection string:
   - Go to Project Settings > Database
   - Copy the connection string

## Step 2: Deploy Backend to Vercel

### Option A: Using Vercel CLI

```bash
cd api
npm install -g vercel
vercel login
vercel deploy
```

### Option B: Using GitHub Integration

1. Push code to GitHub
2. Go to vercel.com and sign in
3. Click "New Project"
4. Select your GitHub repository
5. Configure project settings:
   - Framework: Other
   - Build Command: `npm run build`
   - Output Directory: `dist`

### Step 3: Configure Environment Variables

In Vercel project settings, add:

```
DATABASE_URL=postgresql://user:password@host:5432/database
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_KEY=your-supabase-anon-key
GEMINI_API_KEY=your-gemini-api-key
ELEVENLABS_API_KEY=your-elevenlabs-api-key
EMBEDDING_MODEL=text-embedding-004
LLM_MODEL=gemini-1.5-flash
SIMILARITY_THRESHOLD=0.8
NODE_ENV=production
```

### Step 4: Run Database Migrations

After deployment, run migrations on Supabase:

```bash
# Option 1: Using Supabase CLI
supabase db push

# Option 2: Using SQL Editor
# Copy contents of each migration file and run in Supabase SQL Editor
```

### Step 5: Verify Deployment

```bash
curl https://your-vercel-domain.vercel.app/health
```

Should return:
```json
{
  "status": "ok"
}
```

## Step 6: Deploy Frontend (Optional)

If deploying a frontend:

1. Create a Next.js project
2. Configure API endpoint to point to backend
3. Deploy to Vercel

## Monitoring

### Vercel Logs

View logs in Vercel dashboard:
- Go to your project
- Click "Deployments"
- Select a deployment
- Click "Logs"

### Database Monitoring

Monitor Supabase:
- Go to your project
- Check "Database" section for connection status
- Monitor query performance

## Troubleshooting

### Database Connection Issues

1. Verify DATABASE_URL is correct
2. Check Supabase firewall settings
3. Ensure pgvector extension is enabled
4. Test connection locally first

### API Errors

1. Check Vercel logs for error messages
2. Verify environment variables are set
3. Check external API credentials (Gemini, ElevenLabs)
4. Review request payloads for validation errors

### Performance Issues

1. Check database query performance
2. Monitor connection pool usage
3. Review caching strategies
4. Optimize indexes if needed

## Scaling

### Database Scaling

- Supabase automatically scales PostgreSQL
- Monitor connection pool usage
- Consider read replicas for high traffic

### API Scaling

- Vercel automatically scales serverless functions
- Monitor function execution time
- Optimize long-running operations

## Backup and Recovery

### Database Backups

Supabase provides automatic daily backups:
- Go to Project Settings > Backups
- Configure backup retention
- Download backups if needed

### Disaster Recovery

1. Keep database migrations in version control
2. Document all environment variables
3. Test recovery procedures regularly
4. Maintain database backups

## Security

### SSL/TLS

- Vercel provides automatic SSL certificates
- All traffic is encrypted

### API Keys

- Store all API keys in Vercel environment variables
- Never commit keys to repository
- Rotate keys regularly

### Database Security

- Use strong passwords
- Enable Supabase firewall
- Restrict database access
- Monitor access logs

## Maintenance

### Regular Tasks

- Monitor error rates
- Review performance metrics
- Update dependencies
- Test disaster recovery

### Updates

1. Test updates locally
2. Deploy to staging environment
3. Monitor for issues
4. Deploy to production

## Support

For issues:
1. Check Vercel documentation
2. Check Supabase documentation
3. Review API logs
4. Contact support teams
