#include <vector>
#include <ranges>

#include "ffi.h"
#include "solver.h"

using namespace std::ranges;

template<typename T, typename N>
class RungeKuttaSolver final : public Solver<T, N> {
    T h; // TODO: pass it from Rust somehow
    T current_time;
    std::vector<N> last_solution;
    std::vector<N> buffer;

public:
    explicit RungeKuttaSolver(T step = 0.1) {
        this->h = step;
    }

    void prepare_for_task(CauchyTask<T, N> task) override {
        auto view = task.get_initial_conditions();
        last_solution = std::vector<N>{view.begin(), view.end()};
        current_time = task.get_initial_time();
    }

    N *next_solution(CauchyTask<T, N> task, T &out_time) override {
        auto view = task.get_functions();

        for (std::size_t i = 0; i < view.size(); i++) {
            auto &f = view[i];
            N k1, k2, k3, k4;
            buffer = last_solution;

            k1 = f(current_time, buffer.data());

            for (std::size_t j = 0; j < view.size(); j++)
                buffer[j] = last_solution[j] + k1 * h / 2;
            k2 = f(current_time + h / 2, buffer.data());

            for (std::size_t j = 0; j < view.size(); j++)
                buffer[j] = last_solution[j] + k2 * h / 2;
            k3 = f(current_time + h / 2, buffer.data());

            for (std::size_t j = 0; j < view.size(); j++)
                buffer[j] = last_solution[j] + k3 * h;
            k4 = f(current_time + h, buffer.data());

            last_solution[i] += h / 6 * (k1 + 2 * k2 + 2 * k3 + k4);
        }

        current_time += h;
        out_time = current_time;
        return last_solution.data();
    }
};

template<typename T, typename N>
static RungeKuttaSolver<T, N> GLOBAL_SOLVER;

extern "C" double *solver_eval_next(
    CauchyTask<double, double> task,
    double *out_time
) {
    return GLOBAL_SOLVER<double, double>.next_solution(task, *out_time);
}

extern "C" void solver_prepare(
    CauchyTask<double, double> task
) {
    GLOBAL_SOLVER<double, double>.prepare_for_task(task);
}
