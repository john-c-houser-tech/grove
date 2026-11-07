#pragma once

#include <algorithm>

// Small curves shared by the sky, the weather and the sounds.

// v held between 0 and 1.
inline double clamp01(double v) { return std::clamp(v, 0.0, 1.0); }

// 0 up to a, 1 from b, and an S-shaped ease between.
inline double smoothstep(double a, double b, double x) {
    const double t = clamp01((x - a) / (b - a));
    return t * t * (3 - 2 * t);
}
