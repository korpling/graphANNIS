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

#include "threadpool.h"

using namespace annis;

ThreadPool::ThreadPool(size_t numOfThreads)
  : tasksClosed(false)
{
  worker.reserve(numOfThreads);

  for(size_t i=0; i < numOfThreads; i++)
  {
    worker.emplace_back([this]()
    {

      std::function<void()> f;
      while(true)
      {
        // test if there is a new task available or if the task list was closed
        {
          std::unique_lock<std::mutex> lock(this->mutex_tasks);

          // only wait if the task list is empty right now
          if(!this->tasksClosed && this->tasks.empty())
          {
            this->cond_tasks.wait(lock, [this] {return this->tasksClosed || !this->tasks.empty();});
          }
          if(this->tasksClosed)
          {
            return;
          }


          f = std::move(this->tasks.front());
          this->tasks.pop_front();
        }
        f();

      }
    });
  }
}

annis::ThreadPool::~ThreadPool()
{
  {
    std::lock_guard<std::mutex> lock(mutex_tasks);
    tasksClosed = true;
    tasks.clear();

    cond_tasks.notify_all();
  }

  // make sure each thread is actually finished
  for(size_t i=0; i < worker.size(); i++)
  {
    if(worker[i].joinable())
    {
      worker[i].join();
    }
  }
}
