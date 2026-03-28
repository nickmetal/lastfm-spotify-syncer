variable "project_name" {
  description = "The display name of the GCP project"
  type        = string
  default     = "rsyncer"
}

variable "project_id" {
  description = "The unique project ID (will be suffixed with random string if not unique)"
  type        = string
  default     = "rsyncer"
}

variable "billing_account" {
  description = "The billing account ID to link to the project"
  type        = string
}

variable "org_id" {
  description = "The organization ID (optional, leave empty for personal projects)"
  type        = string
  default     = ""
}

variable "folder_id" {
  description = "The folder ID to create the project in (optional)"
  type        = string
  default     = ""
}

variable "region" {
  description = "The default region for resources"
  type        = string
  default     = "europe-central2"
}

variable "project_owner_email" {
  description = "Email of the GCP project owner to add"
  type        = string
}

variable "monthly_budget_amount" {
  description = "Monthly budget limit in USD"
  type        = number
  default     = 5
}

variable "budget_alert_emails" {
  description = "List of emails to notify when budget thresholds are reached"
  type        = list(string)
  default     = []
}

variable "cloud_run_service_name" {
  description = "Name of the Cloud Run service"
  type        = string
  default     = "rsyncer-api"
}

variable "cloud_run_image" {
  description = "Container image for Cloud Run (use placeholder until image is built)"
  type        = string
  default     = "gcr.io/cloudrun/hello" # Placeholder image
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "labels" {
  description = "Labels to apply to all resources"
  type        = map(string)
  default = {
    app        = "rsyncer"
    managed_by = "terraform"
  }
}
