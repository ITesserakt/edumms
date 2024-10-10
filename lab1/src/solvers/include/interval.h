#pragma once
#include <algorithm>

template<typename T>
struct Interval {
    T start;
    T end;

    explicit Interval(): start(0), end(0) {
    }

    Interval(T value): start(value), end(value) {
    }

    Interval(T start, T end) : start(start), end(end) {
    }

    Interval operator+(const Interval &other) const {
        return Interval(start + other.start, end + other.end);
    }

    Interval operator-(const Interval &other) const {
        return Interval(start - other.start, end - other.end);
    }

    Interval operator*(const Interval &other) const {
        auto [a, b] = std::minmax({start * other.start, start * other.end, end * other.start, end * other.end});
        return Interval(a, b);
    }

    friend Interval operator*(T a, Interval i) {
        return Interval(a * i.start, a * i.end);
    }

    Interval operator*(T other) {
        return Interval(start * other, end * other);
    }

    Interval operator/(const Interval &other) const {
        auto [a, b] = std::minmax({start / other.start, start / other.end, end / other.start, end / other.end});
        return Interval(a, b);
    }
};
