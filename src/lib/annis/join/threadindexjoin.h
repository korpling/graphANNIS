#pragma once

#include <annis/types.h>
#include <annis/iterators.h>

#include <annis/util/sharedqueue.h>
#include <annis/util/threadpool.h>

#include <boost/lockfree/queue.hpp>
#include <thread>
#include <mutex>
#include <atomic>

#include <list>
#include <vector>

namespace annis
{

class Operator;


class ThreadIndexJoin : public Iterator
{
public:
  struct MatchPair
  {
    Match lhs;
    Match rhs;
  };


public:
  ThreadIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            std::shared_ptr<Operator> op,
            std::function<std::list<Annotation>(nodeid_t)> matchGeneratorFunc,
            size_t numOfTasks = 1,
            std::shared_ptr<ThreadPool> threadPool = std::shared_ptr<ThreadPool>());

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~ThreadIndexJoin();
private:

  std::shared_ptr<Iterator> lhs;
  std::mutex mutex_lhs;

  std::shared_ptr<Operator> op;



  std::atomic_bool runBackgroundThreads;
  size_t activeBackgroundTasks;
  std::mutex mutex_activeBackgroundTasks;
  const size_t numOfTasks;
  std::shared_ptr<ThreadPool> threadPool;

  std::unique_ptr<SharedQueue<std::vector<Match>>> results;
  std::function<void()> lhsFetchLoop;

private:
  bool nextLHS(std::vector<Match>& tuple)
  {
    std::lock_guard<std::mutex> lock(mutex_lhs);
    return lhs->next(tuple);
  }

};
}

