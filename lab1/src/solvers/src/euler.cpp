#include <vector>

#include "ffi.h"
#include "solver.h"

template<typename T, typename N>
class EulerSolver : public Solver<T, N> {
    T h;
    T current_time;
    std::vector<N> last_solution;

public:
    explicit EulerSolver(T step = 0.1) {
        this->h = step;
    }

    void prepare_for_task(CauchyTask<T, N> task) override {
        auto view = task.get_initial_conditions();
        current_time = task.get_initial_time();
        last_solution = std::vector<N>{view.begin(), view.end()};
    }

    N *next_solution(CauchyTask<T, N> task, T &out_time) override {
        auto view = task.get_functions();
        auto size = view.size();
        auto result = new N[size];

        for (std::size_t i = 0; i < size; i++) {
            auto &f = view[i];
            result[i] = last_solution[i] + h * f(current_time, last_solution.data());
        }

        current_time += h;
        out_time = current_time;
        std::copy(std::make_move_iterator(result), std::make_move_iterator(result + size), last_solution.begin());
        delete[] result;
        return last_solution.data();
    }
};

template<typename T, typename N>
static EulerSolver<T, N> GLOBAL_SOLVER;

extern "C" double *solver_eval_next(CauchyTask<double, double> task, double *out_time) {
    return GLOBAL_SOLVER<double, double>.next_solution(task, *out_time);
}

extern "C" void solver_prepare(CauchyTask<double, double> task) {
    GLOBAL_SOLVER<double, double>.prepare_for_task(task);
}
