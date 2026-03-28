# Cloud Operations (Monitoring, Logging, Trace)
# Based on GCP Calculator: ~$0.07/month for operations

# Log sink for Cloud Run logs (uses default routing - no extra cost for small volumes)
# Cloud Logging has 50 GiB free ingestion per month

# Uptime check for Cloud Run service (optional - first 3 checks are free)
# resource "google_monitoring_uptime_check_config" "cloud_run_health" {
#   project      = data.google_project.main.project_id
#   display_name = "${var.cloud_run_service_name}-uptime-check"
#   timeout      = "10s"
#   period       = "300s" # Check every 5 minutes

#   http_check {
#     path         = "/"
#     port         = 443
#     use_ssl      = true
#     validate_ssl = true
#   }

#   monitored_resource {
#     type = "uptime_url"
#     labels = {
#       project_id = data.google_project.main.project_id
#       host       = trimprefix(google_cloud_run_v2_service.main.uri, "https://")
#     }
#   }

#   depends_on = [
#     google_project_service.apis,
#     google_cloud_run_v2_service.main
#   ]
# }

# # Alert policy for uptime check failures
# resource "google_monitoring_alert_policy" "uptime_alert" {
#   project      = data.google_project.main.project_id
#   display_name = "${var.cloud_run_service_name}-uptime-alert"
#   combiner     = "OR"

#   conditions {
#     display_name = "Uptime check failure"

#     condition_threshold {
#       filter          = "resource.type = \"uptime_url\" AND metric.type = \"monitoring.googleapis.com/uptime_check/check_passed\""
#       duration        = "300s"
#       comparison      = "COMPARISON_LT"
#       threshold_value = 1

#       trigger {
#         count = 1
#       }

#       aggregations {
#         alignment_period   = "300s"
#         per_series_aligner = "ALIGN_NEXT_OLDER"
#       }
#     }
#   }

#   # Alert strategy
#   alert_strategy {
#     auto_close = "604800s" # Auto-close after 7 days
#   }

#   # Notification channels can be added here
#   # notification_channels = [google_monitoring_notification_channel.email.name]

#   depends_on = [google_project_service.apis]
# }

# Log-based metric for error counting
resource "google_logging_metric" "error_count" {
  project     = data.google_project.main.project_id
  name        = "${var.cloud_run_service_name}_errors"
  description = "Count of error-level log entries for ${var.cloud_run_service_name}"

  filter = <<-EOT
    resource.type="cloud_run_revision"
    resource.labels.service_name="${var.cloud_run_service_name}"
    severity>=ERROR
  EOT

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "INT64"
    unit        = "1"
  }

  depends_on = [google_project_service.apis]
}
