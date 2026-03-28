# Cloud Run service
resource "google_cloud_run_v2_service" "main" {
  project  = data.google_project.main.project_id
  name     = var.cloud_run_service_name
  location = var.region

  # Ensure APIs are enabled first
  depends_on = [google_project_service.apis]

  template {
    labels = {
      # Extract tag after last colon (:) in the image string
      version = regex("([A-Za-z0-9_.-]+)$", var.cloud_run_image)[0]
    }
    service_account = google_service_account.cloud_run.email

    containers {
      image = var.cloud_run_image

      resources {
        limits = {
          cpu    = "1"     # Closest to 0.9 vCPU (Cloud Run uses whole numbers or fractions like "0.5", "1", "2")
          memory = "512Mi" # 512 MiB as per calculator
        }
        cpu_idle = true # Allow CPU to be throttled when idle (cost optimization)
      }

      # Environment variables - add your app-specific vars here
      env {
        name  = "RUST_LOG"
        value = "info"
      }

      # Port configuration
      ports {
        container_port = 8080
      }

      # Startup probe
      startup_probe {
        http_get {
          path = "/health"
        }
        initial_delay_seconds = 0
        timeout_seconds       = 1
        period_seconds        = 240 # 4 minutes (Cloud Run max)
        failure_threshold     = 3
      }
    }

    # Scaling configuration based on calculator:
    # - 0 min instances (scale to zero)
    # - 1 concurrent request per instance
    scaling {
      min_instance_count = 0 # Scale to zero when not in use
      max_instance_count = 5 # Limit max instances for cost control
    }

    # Request timeout: 200ms (from calculator) - but Cloud Run minimum is 1s
    timeout = "60s" # Set reasonable timeout for API calls

    # Max concurrent requests per instance
    max_instance_request_concurrency = 1 # As per calculator
  }

  # Traffic configuration - all traffic to latest revision
  traffic {
    type    = "TRAFFIC_TARGET_ALLOCATION_TYPE_LATEST"
    percent = 100
  }

  labels = local.labels

  # Ignore image changes - Cloud Build manages deployments
  lifecycle {
    ignore_changes = [
      template[0].containers[0].image,
      client,
      client_version,
    ]
  }
}

# IAM binding for allowed users (removes public access)
resource "google_cloud_run_v2_service_iam_binding" "invoker" {
  name    = google_cloud_run_v2_service.main.id
  role    = "roles/run.invoker"
  members = [for email in var.cloud_run_allowed_users : "user:${email}"]
}

## Public access is disabled. Only users in var.cloud_run_allowed_users can invoke the service.
