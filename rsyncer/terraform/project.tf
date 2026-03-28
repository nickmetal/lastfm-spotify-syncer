# Reference existing GCP project (created manually)
data "google_project" "main" {
  project_id = var.project_id
}

# Enable required APIs
resource "google_project_service" "apis" {
  for_each = toset([
    "run.googleapis.com",                  # Cloud Run
    "cloudbuild.googleapis.com",           # Cloud Build
    "cloudresourcemanager.googleapis.com", # Resource Manager
    "iam.googleapis.com",                  # IAM
    "storage.googleapis.com",              # Cloud Storage
    "firestore.googleapis.com",            # Firestore
    "logging.googleapis.com",              # Cloud Logging
    "monitoring.googleapis.com",           # Cloud Monitoring
    "cloudtrace.googleapis.com",           # Cloud Trace
    "cloudbilling.googleapis.com",         # Cloud Billing
    "billingbudgets.googleapis.com",       # Billing Budgets
    "secretmanager.googleapis.com",        # Secret Manager (for future use)
    "artifactregistry.googleapis.com",     # Artifact Registry (for container images)
  ])

  project = data.google_project.main.project_id
  service = each.key

  disable_dependent_services = false
  disable_on_destroy         = false
}
