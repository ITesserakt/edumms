#include <solver.h>
#include <vector>

template<typename T, typename N>
class AdamsBashforth : public Solver<T, N> {
    T h;
    T current_time[2];
    std::vector<N> last_solution[2];

public:
    explicit AdamsBashforth(T step = 0.1) {
        this->h = step;
    }

    void prepare_for_task(CauchyTask<T, N> task) override {
        auto cond_view = task.get_initial_conditions();
        current_time[0] = task.get_initial_time();
        last_solution[0] = {cond_view.begin(), cond_view.end()};

        // Do one step with Euler method to populate last_solution[1]
        auto view = task.get_functions();
        last_solution[1].resize(view.size());
        for (std::size_t i = 0; i < view.size(); ++i) {
            auto &f = view[i];
            last_solution[1][i] = last_solution[0][i] + h * f(current_time[0], last_solution[0].data());
        }
        current_time[1] = current_time[0] + h;
    }

    N *next_solution(CauchyTask<T, N> task, T &out_time) override {
        auto view = task.get_functions();
        auto size = view.size();
        std::vector<N> result;
        result.resize(size);

        for (std::size_t i = 0; i < size; i++) {
            auto &f = view[i];

            // y n = y n-1 + h(1.5 f(x n-1, y n-1) - 0.5 f(x n-2, y n-2))
            result[i] = last_solution[1][i] + h * (3. / 2. * f(current_time[1], last_solution[1].data()) -
                                                   1. / 2. * f(current_time[0], last_solution[0].data()));
        }

        current_time[0] = current_time[1];
        current_time[1] += h;
        out_time = current_time[1];
        // move y n-1 to y n-2
        last_solution[0].swap(last_solution[1]);
        // move result to y n-1
        last_solution[1].swap(result);
        return last_solution[1].data();
    }
};

template<typename T, typename N>
static AdamsBashforth<T, N> GLOBAL_SOLVER = AdamsBashforth<T, N>();

extern "C" double *solver_eval_next(CauchyTask<double, double> task, double *out_time) {
    return GLOBAL_SOLVER<double, double>.next_solution(task, *out_time);
}

extern "C" void solver_prepare(CauchyTask<double, double> task) {
    GLOBAL_SOLVER<double, double>.prepare_for_task(task);
}
