#include "threadpool.h"

using namespace annis;

ThreadPool::ThreadPool(size_t numOfThreads)
  : stopped(false), tasks(128)
{
  worker.reserve(numOfThreads);

  for(size_t i=0; i < numOfThreads; i++)
  {
    worker.emplace_back([this]()
    {

      std::function<void()> f;
      while(!this->stopped)
      {
        // test if there is a new task available
        while(!this->stopped && this->tasks.try_dequeue(f))
        {
          // execute this task
          f();
        }

        // wait until a new task is available before trying again
        {
          std::unique_lock<std::mutex> lock(mutex_Global);
          cond_NewItem.wait(lock);
        }
      } // end while not stopped
    });
  }
}

annis::ThreadPool::~ThreadPool()
{
  {
    // wait until everyone is waiting
    std::lock_guard<std::mutex> lock(mutex_Global);
    stopped = true;
  }
  cond_NewItem.notify_all();

  // make sure each thread is actually finished
  for(size_t i=0; i < worker.size(); i++)
  {
    worker[i].join();
  }
}
