# Budget alert to prevent spending more than $5/month
# NOTE: Commented out due to billing API permission issues with ADC.
# Create the budget manually in the GCP Console:
# https://console.cloud.google.com/billing/budgets
#
# resource "google_billing_budget" "monthly" {
#   billing_account = var.billing_account
#   display_name    = "${var.project_name} Monthly Budget"
#
#   budget_filter {
#     projects               = ["projects/${data.google_project.main.number}"]
#     credit_types_treatment = "INCLUDE_ALL_CREDITS"
#   }
#
#   amount {
#     specified_amount {
#       currency_code = "USD"
#       units         = tostring(var.monthly_budget_amount)
#     }
#   }
#
#   # Alert thresholds
#   threshold_rules {
#     threshold_percent = 0.5 # 50% - Alert at $2.50
#     spend_basis       = "CURRENT_SPEND"
#   }
#
#   threshold_rules {
#     threshold_percent = 0.8 # 80% - Alert at $4.00
#     spend_basis       = "CURRENT_SPEND"
#   }
#
#   threshold_rules {
#     threshold_percent = 1.0 # 100% - Alert at $5.00
#     spend_basis       = "CURRENT_SPEND"
#   }
#
#   threshold_rules {
#     threshold_percent = 1.2 # 120% - Alert at $6.00 (over budget)
#     spend_basis       = "CURRENT_SPEND"
#   }
#
#   # Email notifications
#   all_updates_rule {
#     monitoring_notification_channels = []
#
#     # Send to billing admins
#     disable_default_iam_recipients = false
#
#     # Additional schema version for pub/sub notifications (optional)
#     schema_version = "1.0"
#   }
#
#   depends_on = [google_project_service.apis]
# }

# Note: To receive email notifications, you need to:
# 1. Create a monitoring notification channel for email
# 2. Add the channel to monitoring_notification_channels above
#
# Alternatively, budget alerts are automatically sent to:
# - Billing account administrators
# - Billing account users
#
# To add custom email recipients, create a notification channel like this:
#
# resource "google_monitoring_notification_channel" "budget_email" {
#   project      = data.google_project.main.project_id
#   display_name = "Budget Alerts Email"
#   type         = "email"
#   labels = {
#     email_address = "your-email@example.com"
#   }
# }
