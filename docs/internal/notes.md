# Unit Behavior

## Capitals

### Objective

Units can have one of the following movement objectives, in order of priority:
- Follow a path
    - Move to a point or a collection of points
    - Assigning a path clears the Priority Target List
- Follow unit
    - Target (Pursue top of Priority Target List: Move to within range of enemy)
    - Guard (Follow on Friendly: track a unit closely)
    * Advanced UI: choose Follow distance (versus weapon range)

Action objectives

### Control


# Weapon system

## Weapon types

- Point defense cannons
    - Small, short-range, mid-speed projectiles effective against low HP targets, very ineffective against higher HP targets
- Ballistic cannons
    - Effective against all armor types equally. Fast tracking turrets, but slow projectile speed. Can be shot down by PDL.
- Coilguns
    - Fast projectiles, mid range, slow turrets (cannot track faster strike craft and caps)
- Railguns
    - Fastest projectiles, longest range cannons, slowest turrets (or fixed direction on caps)
- Missiles
    - Variety of speeds, lifetimes and nimbleness. Can be shot down by projectiles.

## Hitpoints types
- HP
    - Armor
- Shield
    - Shield generators are a subunit which add additional hitpoints

## Other
- PDL (Point defense lasers)
    - Shoot down missiles and ballistic cannons


# Subunits

- Turrets
- Foward weapons
- Shield generator
- Thruster
- Compartment

# Physics

## Capital Ship Drag vs. Distance to Destination:

All bodies, like capital ships, have a velocity and a position.

Ships accelerate in the direction of their targets each frame.

While capital ships accelerate slowly, they can reach high speeds in order to traverse the great distances of the solar system.

Their velocity *v* is scaled by some fraction of its current value to simulate course adjustments and prevent ship speeds from growing without bound.
$$ v = v_{-1} * d $$

*d* is bound between 0.0 and 1.0 and grows smaller relative to the distance of the ship to its target and the angle between the ship's current heading and its target.

Where *r* is the distance from the ship to its destination and *w* is relative to the dot product between the ship's current heading *A* and the target heading *B*: 

$$ w = \frac{1 - A \cdot B}{2} $$

$$ d\ =\left(\frac{x\left(1.01-w\right)}{r\ }\right)^{\frac{1}{100}} $$

https://www.desmos.com/calculator/yr7xmqzpbh

