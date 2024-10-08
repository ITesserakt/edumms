#pragma once
#include <cstddef>

template <typename T, typename N>
using FFIClosure = N *(void *, T, N[], std::size_t);

template<typename T, typename N>
struct FFICauchyTask {
    std::size_t size;
    T initial_time;
    FFIClosure<T, N> *derivatives;
    N *initial_conditions;
};


