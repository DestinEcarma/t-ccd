"""Configuration constants for T-CCD validation framework."""

# Simulation constants
time_step = 1.0 / 60.0  # 60 FPS simulation
max_frames = 18000
output_dir = "brute_force_validation"
chunk_size = 100000
collision_tolerance = 1e-10
progress_report_interval = 10

# File paths (use Swept AABB particles as ground truth for validation)
particles_tccd_file = "data/particles_tccd_500.csv"
particles_swept_aabb_file = "data/particles_swept_aabb_500.csv"
tccd_events_file = "data/events_tccd_500.csv"
swept_events_file = "data/events_swept_aabb_500.csv"