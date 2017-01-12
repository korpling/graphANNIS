/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#pragma once

#include <thread>
#include <future>
#include <deque>
#include <vector>

namespace annis
{

class ThreadPool
{
public:

  ThreadPool(size_t numOfThreads);

  template<class F, class... Args>
  std::future<typename std::result_of<F(Args...)>::type> enqueue(F&& f, Args&&... args)
  {
    using return_type = typename std::result_of<F(Args...)>::type;

    auto newTask = std::make_shared< std::packaged_task<return_type()> >(
                std::bind(std::forward<F>(f), std::forward<Args>(args)...)
    );

    std::future<return_type> res = newTask->get_future();

    {
      std::lock_guard<std::mutex> lock(mutex_tasks);
      tasks.emplace_back([newTask](){ (*newTask)(); });

    }
    cond_tasks.notify_one();

    return res;
  }

  ~ThreadPool();

private:

  bool tasksClosed;
  std::deque<std::function<void()>> tasks;
  std::mutex mutex_tasks;
  std::condition_variable cond_tasks;


  std::vector<std::thread> worker;


};
} // end namespace annis
