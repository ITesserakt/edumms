#pragma once

#include "ffi.h"

template<typename T, typename N>
struct Solver {
    virtual ~Solver() = default;

    virtual void prepare_for_task(CauchyTask<T, N> task) = 0;
    virtual N* next_solution(CauchyTask<T, N> task, T& out_time) = 0;
};