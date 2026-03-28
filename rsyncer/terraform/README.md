# Terraform Configuration for rsyncer

This Terraform configuration manages GCP resources for the rsyncer application.

**Note:** Due to GCP permission restrictions, the project must be created manually before running Terraform.

## Resources Created

| Resource | Description | Monthly Cost Estimate |
|----------|-------------|----------------------|
| GCP Project | **Created manually** (see setup) | $0.00 |
| Artifact Registry | Docker image repository with cleanup policies | ~$0.00 (first 500MB free) |
| Cloud Run | Serverless container (europe-central2, 1 vCPU, 512 MiB) | ~$0.03 |
| Cloud Storage | Data bucket with lifecycle policies | ~$0.04 |
| Firestore | Native mode database | ~$0.00 (free tier) |
| Cloud Operations | Logging, Monitoring, Trace | ~$0.07 |
| Budget Alert | **Created manually** (see setup) | $0.00 |
| **Total** | | **~$0.15/month** |

## Prerequisites

1. [Terraform](https://www.terraform.io/downloads) >= 1.14.0
2. [Google Cloud SDK](https://cloud.google.com/sdk/docs/install)
3. A GCP billing account

## Setup

### Step 1: Create GCP Project Manually

Due to organization restrictions, the project must be created via the GCP Console:

1. Go to https://console.cloud.google.com/projectcreate
2. Create a project (e.g., `rsyncer`)
3. Note the **Project ID** (e.g., `rsyncer-491618`)
4. Link a billing account to the project in the Console

### Step 2: Authenticate with GCP

```bash
# Login to GCP
gcloud auth login

# Set up Application Default Credentials
gcloud auth application-default login

# Set the quota project (use your project ID)
gcloud auth application-default set-quota-project YOUR_PROJECT_ID

# Set the default project
gcloud config set project YOUR_PROJECT_ID
```

### Step 3: Grant Billing Permissions (if needed)

```bash
# Get your billing account ID
gcloud billing accounts list

# Grant yourself billing user role (if you get permission errors)
gcloud billing accounts add-iam-policy-binding YOUR_BILLING_ACCOUNT_ID \
  --member="user:YOUR_EMAIL" \
  --role="roles/billing.user"
```

### Step 4: Enable Required APIs

```bash
# Enable billing budgets API (required for budget alerts)
gcloud services enable billingbudgets.googleapis.com --project=YOUR_PROJECT_ID
```

### Step 5: Configure Terraform

```bash
# Copy the example tfvars file
cp terraform.tfvars.example ../terraform.tfvars

# Edit with your values - IMPORTANT: set project_id to your existing project
# project_id = "rsyncer-491618"  # Your actual project ID
```

### Step 6: Run Terraform

```bash
terraform init
terraform plan -var-file ../terraform.tfvars
terraform apply -var-file ../terraform.tfvars
```

### Step 7: Create Budget Alert Manually

Due to ADC permission issues with the billing budgets API, create the budget manually:

1. Go to https://console.cloud.google.com/billing/budgets
2. Click **Create Budget**
3. Set:
   - Budget name: `rsyncer Monthly Budget`
   - Projects: Select your project
   - Amount: $5
   - Thresholds: 50%, 80%, 100%
4. Configure email notifications as needed

## Configuration Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `project_id` | Yes | - | Existing GCP project ID (created manually) |
| `billing_account` | Yes | - | Billing account ID |
| `project_owner_email` | Yes | - | Email of project owner |
| `org_id` | No | `""` | Organization ID |
| `folder_id` | No | `""` | Folder ID |
| `region` | No | `europe-central2` | GCP region |
| `monthly_budget_amount` | No | `5` | Monthly budget in USD |
| `cloud_run_image` | No | `gcr.io/cloudrun/hello` | Container image |

## Outputs

After applying, you'll get:
- `project_id` - The project ID
- `cloud_run_url` - URL of the Cloud Run service
- `storage_bucket` - Name of the storage bucket
- `useful_commands` - Helpful gcloud commands

## Deploying Your Application

1. Build and push your container image:
   ```bash
   # Using Cloud Build
   gcloud builds submit --tag gcr.io/$(terraform output -raw project_id)/rsyncer

   # Or using Artifact Registry
   gcloud builds submit --tag $(terraform output -raw region)-docker.pkg.dev/$(terraform output -raw project_id)/rsyncer/rsyncer
   ```

2. Update the Cloud Run service:
   ```bash
   terraform apply -var="cloud_run_image=gcr.io/YOUR_PROJECT/rsyncer:latest"
   ```

## Cost Management

- Budget alert is configured at $5/month
- Alerts at 50%, 80%, 100%, and 120% of budget
- Cloud Run scales to zero when not in use
- Storage uses lifecycle policies to reduce costs

## Cleanup

To destroy all resources:
```bash
terraform destroy
```

**Warning:** This will delete all data in Cloud Storage and Firestore!
