terraform {
  required_version = ">= 1.14.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 7.25"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "~> 7.25"
    }
  }
}
