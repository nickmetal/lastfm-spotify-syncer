# Artifact Registry repository for container images
resource "google_artifact_registry_repository" "rsyncer_api" {
  project       = data.google_project.main.project_id
  location      = var.region
  repository_id = "rsyncer-api"
  description   = "Docker images for rsyncer API"
  format        = "DOCKER"

  # Cleanup policy to limit storage costs
  cleanup_policy_dry_run = false
  cleanup_policies {
    id     = "keep-recent-versions"
    action = "KEEP"
    most_recent_versions {
      keep_count = 10
    }
  }

  cleanup_policies {
    id     = "delete-old-untagged"
    action = "DELETE"
    condition {
      tag_state  = "UNTAGGED"
      older_than = "604800s" # 7 days
    }
  }

  depends_on = [google_project_service.apis]
}
