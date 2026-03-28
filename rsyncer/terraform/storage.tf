# Cloud Storage bucket for application data
resource "google_storage_bucket" "data" {
  project  = data.google_project.main.project_id
  name     = "${var.project_id}-data"
  location = var.region

  # Use Standard storage class (cheapest for frequent access)
  storage_class = "STANDARD"

  # Enable uniform bucket-level access (recommended)
  uniform_bucket_level_access = true

  # Lifecycle rules for cost optimization
  lifecycle_rule {
    condition {
      age = 365 # Move to Nearline after 1 year
    }
    action {
      type          = "SetStorageClass"
      storage_class = "NEARLINE"
    }
  }

  lifecycle_rule {
    condition {
      age = 730 # Move to Coldline after 2 years
    }
    action {
      type          = "SetStorageClass"
      storage_class = "COLDLINE"
    }
  }

  # Enable versioning for data protection
  versioning {
    enabled = true
  }

  # Delete old versions after 30 days
  lifecycle_rule {
    condition {
      num_newer_versions = 3
      with_state         = "ARCHIVED"
    }
    action {
      type = "Delete"
    }
  }

  labels = local.labels

  # Prevent accidental deletion
  lifecycle {
    prevent_destroy = false # Set to true in production
  }

  depends_on = [google_project_service.apis]
}

# Grant Cloud Run service account access to the bucket
resource "google_storage_bucket_iam_member" "cloud_run_access" {
  bucket = google_storage_bucket.data.name
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.cloud_run.email}"
}
