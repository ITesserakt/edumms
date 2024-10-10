#pragma once

#include <memory>

#include "ffi.h"
#include "interval.h"

template<typename T, typename N>
struct Solver {
    virtual ~Solver() = default;

    virtual void prepare_for_task(CauchyTask<T, N> task) = 0;

    virtual N *next_solution(CauchyTask<T, N> task, T &out_time) = 0;
};

/// Defined by header's consumer
template<typename T, typename N>
extern std::unique_ptr<Solver<T, N> > GLOBAL_SOLVER;

#define gen_binding(solver_obj, time_ty, out_ty, suffix)                                                      \
    extern "C" const out_ty* solver_eval_next_##suffix(CauchyTask<time_ty, out_ty> task, time_ty* out_time) { \
        return solver_obj<time_ty, out_ty>->next_solution(task, *out_time);                                   \
    }                                                                                                         \
    extern "C" void solver_prepare_##suffix(CauchyTask<time_ty, out_ty> task) {                               \
        solver_obj<time_ty, out_ty>->prepare_for_task(task);                                                  \
    }

gen_binding(GLOBAL_SOLVER, double, double, f64_f64)
gen_binding(GLOBAL_SOLVER, float, float, f32_f32)
gen_binding(GLOBAL_SOLVER, double, Interval<double>, f64_If64)
gen_binding(GLOBAL_SOLVER, float, Interval<double>, f32_If64)
gen_binding(GLOBAL_SOLVER, double, Interval<float>, f64_If32)
gen_binding(GLOBAL_SOLVER, float, Interval<float>, f32_If32)
