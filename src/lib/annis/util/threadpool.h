#pragma once

#include <thread>
#include <future>
#include <list>
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
      if(!tasksClosed)
      {
        tasks.emplace_back([newTask](){ (*newTask)(); });
      }
    }
    cond_tasks.notify_one();

    return res;
  }

  ~ThreadPool();

private:

  bool tasksClosed;
  std::list<std::function<void()>> tasks;
  std::mutex mutex_tasks;
  std::condition_variable cond_tasks;


  std::vector<std::thread> worker;


};
} // end namespace annis
