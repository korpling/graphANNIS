#pragma once

#include <thread>
#include <future>
#include <vector>

#include <thread>
#include <atomic>

#include <concurrentqueue.h>

namespace annis
{

/**
 *  Thread pool implementation using a lock-free queue for the tasks.
 */
class ThreadPool
{
public:

  ThreadPool(size_t numOfThreads);

  template<class F, class... Args>
  std::future<typename std::result_of<F(Args...)>::type> enqueue(F&& f, Args&&... args)
  {
    using return_type = typename std::result_of<F(Args...)>::type;

    auto task = std::make_shared< std::packaged_task<return_type()> >(
                std::bind(std::forward<F>(f), std::forward<Args>(args)...)
    );

    std::future<return_type> res = task->get_future();

    if(!stopped)
    {
      tasks.enqueue([task](){ (*task)(); });
    }

    {
      std::lock_guard<std::mutex> lock(mutex_Global);
      cond_NewItem.notify_one();
    }

    return res;
  }

  ~ThreadPool();

private:

  std::atomic_bool stopped;

  std::vector<std::thread> worker;
  moodycamel::ConcurrentQueue<std::function<void()>> tasks;

  std::mutex mutex_Global;
  std::condition_variable cond_NewItem;
};
} // end namespace annis
