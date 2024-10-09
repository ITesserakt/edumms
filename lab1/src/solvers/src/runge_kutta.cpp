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
        auto size = view.size();
        auto temp = new N[size * 5];
        auto result = temp, k1 = &temp[size], k2 = &temp[size * 2], k3 = &temp[size * 3], k4 = &temp[size * 4];

        for (std::size_t i = 0; i < size; i++) {
            k1[i] = view[i](current_time, last_solution.data());
            result[i] = last_solution[i] + k1[i] * h / 2.;
        }

        for (std::size_t i = 0; i < size; i++) {
            k2[i] = view[i](current_time + h / 2., temp);
            result[i] = last_solution[i] + k2[i] * h / 2.;
        }

        for (std::size_t i = 0; i < size; i++) {
            k3[i] = view[i](current_time + h / 2., temp);
            result[i] = last_solution[i] + k3[i] * h;
        }

        for (std::size_t i = 0; i < size; i++) {
            k4[i] = view[i](current_time + h, temp);
            result[i] = last_solution[i] + h / 6. * (k1[i] + 2. * k2[i] + 2. * k3[i] + k4[i]);
        }

        current_time += h;
        out_time = current_time;
        std::copy(std::make_move_iterator(temp), std::make_move_iterator(temp + size), last_solution.begin());

        delete[] temp;
        return last_solution.data();
    }
};

template<typename T, typename N>
static RungeKuttaSolver<T, N> GLOBAL_SOLVER;

extern "C" double *solver_eval_next(CauchyTask<double, double> task, double *out_time) {
    return GLOBAL_SOLVER<double, double>.next_solution(task, *out_time);
}

extern "C" void solver_prepare(CauchyTask<double, double> task) {
    GLOBAL_SOLVER<double, double>.prepare_for_task(task);
}
