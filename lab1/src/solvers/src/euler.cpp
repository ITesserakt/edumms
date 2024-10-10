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
        auto view = task.initial_conditions;
        current_time = task.initial_time;
        last_solution = std::vector<N>{view, view + task.size};
    }

    N *next_solution(CauchyTask<T, N> task, T &out_time) override {
        auto view = task.derivatives;
        auto size = task.size;
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
std::unique_ptr<Solver<T, N>> GLOBAL_SOLVER = std::make_unique<EulerSolver<T, N>>();