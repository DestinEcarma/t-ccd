"""Core collision detection algorithms using continuous collision detection."""

import math
from typing import Optional

from ..config import collision_tolerance
from ..models import Collision, Particle


class CollisionDetector:
    """Core collision detection algorithms using continuous collision detection."""

    @staticmethod
    def detect_collision(p1: Particle, p2: Particle, time_step: float) -> Optional[Collision]:
        """
        Detect collision between two particles using continuous collision detection.
        
        Uses quadratic formula to solve for exact collision time:
        |p1(t) - p2(t)| = r1 + r2
        
        Args:
            p1, p2: Particle objects to check for collision
            time_step: Time step size for this frame
            
        Returns:
            Collision object if collision occurs, None otherwise
        """
        # Relative position and velocity
        dx = p1.x - p2.x
        dy = p1.y - p2.y
        dvx = p1.vx - p2.vx
        dvy = p1.vy - p2.vy
        
        # Combined radius
        r_sum = p1.radius + p2.radius
        
        # Quadratic equation coefficients: a*t² + b*t + c = 0
        a = dvx * dvx + dvy * dvy
        b = 2.0 * (dx * dvx + dy * dvy)
        c = dx * dx + dy * dy - r_sum * r_sum
        
        # Check if particles are moving apart
        if b > 0:
            return None
            
        # Solve quadratic equation
        discriminant = b * b - 4.0 * a * c
        
        if discriminant < 0:
            return None
        
        # Calculate collision time
        if abs(a) < collision_tolerance:
            # Particles have same velocity - check if already overlapping
            if c <= 0:
                t = 0.0
            else:
                return None
        else:
            sqrt_d = math.sqrt(discriminant)
            t1 = (-b - sqrt_d) / (2.0 * a)
            t2 = (-b + sqrt_d) / (2.0 * a)
            
            # We want the first collision time (smallest positive t)
            if 0 <= t1 <= time_step:
                t = t1
            elif 0 <= t2 <= time_step:
                t = t2
            else:
                return None
        
        # Calculate collision details
        return CollisionDetector._create_collision_object(p1, p2, t, time_step)
    
    @staticmethod
    def _create_collision_object(p1: Particle, p2: Particle, t: float, time_step: float) -> Collision:
        """Create collision object with calculated collision details."""
        # Collision positions
        col_x1, col_y1 = p1.position_at_time(t)
        col_x2, col_y2 = p2.position_at_time(t)
        
        # Collision position (midpoint between particle centers)
        col_x = (col_x1 + col_x2) / 2.0
        col_y = (col_y1 + col_y2) / 2.0
        
        # Normal vector (from p2 to p1)
        normal_x = col_x1 - col_x2
        normal_y = col_y1 - col_y2
        normal_length = math.sqrt(normal_x**2 + normal_y**2)
        
        if normal_length > collision_tolerance:
            normal_x /= normal_length
            normal_y /= normal_length
        else:
            normal_x, normal_y = 1.0, 0.0
        
        # Calculate relative velocity magnitude
        rel_vx = p1.vx - p2.vx
        rel_vy = p1.vy - p2.vy
        rel_vel = math.sqrt(rel_vx**2 + rel_vy**2)
        
        # Distance between centers at collision
        distance = math.sqrt((col_x1 - col_x2)**2 + (col_y1 - col_y2)**2)
        
        return Collision(
            frame=p1.frame,
            time_s=p1.frame * time_step + t,
            toi=t,
            particle1=p1.particle_id,
            particle2=p2.particle_id,
            x=col_x,
            y=col_y,
            nx=normal_x,
            ny=normal_y,
            distance=distance,
            relative_velocity=rel_vel
        )