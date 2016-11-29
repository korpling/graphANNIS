#pragma once

#include <queue>
#include <mutex>
#include <condition_variable>

namespace annis
{
  /**
   * This is a thread-safe queue that has a blocking pop() function.
   * Additionally it is possible to shutdown a queue. If a queue is shutdown, not new entries
   * can be added and as soon as the queue is empty the pop() funtion will return immediatly instead of waiting forever.
   * A shutdown can't be undone.
   */
  template<typename T>
  class SharedQueue
  {
  public:

    SharedQueue()
    : isShutdown(false)
    {

    }

    /**
     * @brief Retrieve an item from the queue. This will block until an item is available. If the queue is empty
     * and shut-down it will return immediatly with "false" as a result.
     * @param item
     * @return "true" if an item was retrieved from the queue, false if not.
     */
    bool pop(T& item)
    {
      std::unique_lock<std::mutex> lock(queueMutex);
      while(queue.empty())
      {
        if(isShutdown)
        {
          // queue is empty and since it is shut down no new entries will be added.
          return false;
        }
        else
        {
          queueCondition.wait(lock);
        }
      }
      item = queue.front();
      queue.pop();

      return true;
    }

    void push(const T& item)
    {
      std::unique_lock<std::mutex> lock(queueMutex);

      if(!isShutdown)
      {
        queue.push(item);
        lock.unlock();
        queueCondition.notify_one();
      }
    }

    void shutdown()
    {
      std::unique_lock<std::mutex> lock(queueMutex);
      if(!isShutdown)
      {
        isShutdown = true;
        lock.unlock();
        queueCondition.notify_one();
      }
    }


  private:

    std::queue<T> queue;
    bool isShutdown;

    std::mutex queueMutex;
    std::condition_variable queueCondition;
  };
}
