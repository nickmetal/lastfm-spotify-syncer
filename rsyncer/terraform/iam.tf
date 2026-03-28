# Add project owner
resource "google_project_iam_member" "project_owner" {
  project = data.google_project.main.project_id
  role    = "roles/owner"
  member  = "user:${var.project_owner_email}"
}

# Service account for Cloud Run
resource "google_service_account" "cloud_run" {
  project      = data.google_project.main.project_id
  account_id   = "cloud-run-sa"
  display_name = "Cloud Run Service Account"
  description  = "Service account used by Cloud Run service for rsyncer"

  depends_on = [google_project_service.apis]
}

# Grant basic developer permissions to the Cloud Run service account
# These are minimal permissions needed for the application to run

# Allow reading from Cloud Storage
resource "google_project_iam_member" "cloud_run_storage_viewer" {
  project = data.google_project.main.project_id
  role    = "roles/storage.objectViewer"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}

# Allow writing to Cloud Storage
resource "google_project_iam_member" "cloud_run_storage_creator" {
  project = data.google_project.main.project_id
  role    = "roles/storage.objectCreator"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}

# Allow Firestore access
resource "google_project_iam_member" "cloud_run_firestore" {
  project = data.google_project.main.project_id
  role    = "roles/datastore.user"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}

# Allow writing logs
resource "google_project_iam_member" "cloud_run_logging" {
  project = data.google_project.main.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}

# Allow writing metrics
resource "google_project_iam_member" "cloud_run_monitoring" {
  project = data.google_project.main.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}

# Allow writing traces
resource "google_project_iam_member" "cloud_run_trace" {
  project = data.google_project.main.project_id
  role    = "roles/cloudtrace.agent"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}

# Allow reading secrets (for environment variables)
resource "google_project_iam_member" "cloud_run_secret_accessor" {
  project = data.google_project.main.project_id
  role    = "roles/secretmanager.secretAccessor"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}

# Allow Cloud Run to act as this service account
resource "google_project_iam_member" "cloud_run_service_account_user" {
  project = data.google_project.main.project_id
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${google_service_account.cloud_run.email}"
}
