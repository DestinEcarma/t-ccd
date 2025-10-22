"""Data loading utilities for T-CCD validation framework."""

from typing import Dict, List, Tuple

import pandas as pd

from ..config import chunk_size
from ..models import Particle


class DataLoader:
    """Handles loading and parsing of particle and collision data files."""
    
    def __init__(self, 
                 particles_tccd_file: str = "data/particles_tccd_500.csv",
                 particles_swept_aabb_file: str = "data/particles_swept_aabb_500.csv",
                 tccd_events_file: str = "data/events_tccd_500.csv",
                 swept_events_file: str = "data/events_swept_aabb_500.csv"):
        self.particles_tccd_file = particles_tccd_file
        self.particles_swept_aabb_file = particles_swept_aabb_file
        self.tccd_events_file = tccd_events_file
        self.swept_events_file = swept_events_file
    
    def load_particle_states(self, use_tccd_particles: bool = False, max_frames: int = None) -> Dict[int, List[Particle]]:
        """
        Load particle states from CSV files.
        
        Args:
            use_tccd_particles: If True, use T-CCD particles; if False, use Swept AABB particles
            max_frames: Maximum number of frames to load
        
        Returns:
            Dict[frame_number] -> List[Particle]
        """
        particles_file = self.particles_tccd_file if use_tccd_particles else self.particles_swept_aabb_file
        source_name = "T-CCD" if use_tccd_particles else "Swept AABB"
        
        print(f"📊Loading {source_name} particle states from: {particles_file}")
        
        particle_states = {}
        
        try:
            for chunk in pd.read_csv(particles_file, chunksize=chunk_size):
                for _, row in chunk.iterrows():
                    frame = int(row['frame'])
                    
                    # Apply frame limit
                    if max_frames is not None and frame > max_frames:
                        break
                        
                    if frame not in particle_states:
                        particle_states[frame] = []
                    
                    particle = Particle(
                        particle_id=int(row['particle_id']),
                        frame=frame,
                        x=float(row['x']),
                        y=float(row['y']),
                        vx=float(row['vx']),
                        vy=float(row['vy']),
                        radius=float(row['radius']),
                        mass=float(row.get('mass', 1.0))
                    )
                    
                    particle_states[frame].append(particle)
                
                # Check if we've reached the frame limit
                if max_frames is not None and particle_states and max(particle_states.keys()) >= max_frames:
                    break
            
            if particle_states:
                frames = sorted(particle_states.keys())
                particle_count = len(particle_states[frames[0]]) if frames else 0
                print(f" Loaded {len(particle_states)} frames with ~{particle_count} particles each")
                print(f" Frame range: {min(frames)} to {max(frames)}")
            else:
                print(" No particle states loaded")
            
            return particle_states
            
        except Exception as e:
            print(f" Error loading particles: {e}")
            return {}
    
    def load_algorithm_events(self, max_frames: int = None) -> Tuple[pd.DataFrame, pd.DataFrame]:
        """Load T-CCD and Swept AABB collision events."""
        print("🔍 Loading algorithm collision events...")
        
        try:
            # Load T-CCD events
            tccd_events = pd.read_csv(self.tccd_events_file)
            if max_frames is not None:
                tccd_events = tccd_events[tccd_events['frame'] <= max_frames]
            print(f"    T-CCD events: {len(tccd_events)} collisions")
            
            # Load Swept AABB events
            swept_events = pd.read_csv(self.swept_events_file)
            if max_frames is not None:
                swept_events = swept_events[swept_events['frame'] <= max_frames]
            print(f"    Swept AABB events: {len(swept_events)} collisions")
            
            return tccd_events, swept_events
            
        except Exception as e:
            print(f" Error loading algorithm events: {e}")
            return pd.DataFrame(), pd.DataFrame()