# Firestore database in Native mode
resource "google_firestore_database" "main" {
  project     = data.google_project.main.project_id
  name        = "(default)"
  location_id = var.region
  type        = "FIRESTORE_NATIVE"

  # Enable delete protection in production
  delete_protection_state = "DELETE_PROTECTION_DISABLED"

  # Point-in-time recovery
  point_in_time_recovery_enablement = "POINT_IN_TIME_RECOVERY_DISABLED" # Enable if needed (has cost)

  depends_on = [google_project_service.apis]
}

# Note: Firestore has a free tier:
# - 1 GiB storage
# - 50,000 document reads per day
# - 20,000 document writes per day
# - 20,000 document deletes per day
# This should be sufficient for the rsyncer application
