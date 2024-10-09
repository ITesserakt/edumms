cmake_minimum_required(VERSION 3.5)

project(solvers)

set(CMAKE_CXX_FLAGS "-Wall -Wextra -Wpedantic -std=c++20 -g")

add_library(runge-kutta SHARED src/runge_kutta.cpp)
target_include_directories(runge-kutta PUBLIC include/)

add_library(euler SHARED src/euler.cpp)
target_include_directories(euler PUBLIC include/)
