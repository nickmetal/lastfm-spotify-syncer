# Output values

output "project_id" {
  description = "The GCP project ID"
  value       = data.google_project.main.project_id
}

output "project_number" {
  description = "The GCP project number"
  value       = data.google_project.main.number
}

output "cloud_run_url" {
  description = "The URL of the Cloud Run service"
  value       = google_cloud_run_v2_service.main.uri
}

output "cloud_run_service_account" {
  description = "The service account used by Cloud Run"
  value       = google_service_account.cloud_run.email
}

output "storage_bucket" {
  description = "The Cloud Storage bucket name"
  value       = google_storage_bucket.data.name
}

output "firestore_database" {
  description = "The Firestore database name"
  value       = google_firestore_database.main.name
}

output "region" {
  description = "The region where resources are deployed"
  value       = var.region
}

output "artifact_registry_url" {
  description = "The Artifact Registry URL for container images"
  value       = "${var.region}-docker.pkg.dev/${data.google_project.main.project_id}/${google_artifact_registry_repository.rsyncer_api.repository_id}"
}

# Useful commands output
output "useful_commands" {
  description = "Helpful gcloud commands for this project"
  value       = <<-EOT

    # Set project as default
    gcloud config set project ${data.google_project.main.project_id}

    # Build and push image with Cloud Build
    gcloud builds submit --config=cloudbuild.yaml .

    # View Cloud Run logs
    gcloud run services logs read ${var.cloud_run_service_name} --project=${data.google_project.main.project_id} --region=${var.region}

    # Update Cloud Run service with new image
    gcloud run deploy ${var.cloud_run_service_name} --image=${var.region}-docker.pkg.dev/${data.google_project.main.project_id}/${google_artifact_registry_repository.rsyncer_api.repository_id}/rsyncer-api:latest --project=${data.google_project.main.project_id} --region=${var.region}

    # View billing
    gcloud billing projects describe ${data.google_project.main.project_id}

  EOT
}
