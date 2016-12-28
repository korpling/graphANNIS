#pragma once

#include <annis/types.h>
#include <annis/iterators.h>

#include <annis/util/comparefunctions.h>

#include <boost/lockfree/queue.hpp>
#include <thread>
#include <future>

#include <list>

#include <annis/util/threadpool.h>


namespace annis
{

class Operator;


class TaskIndexJoin : public Iterator
{
public:

  struct MatchPair
  {
    std::vector<Match> lhs;
    Match rhs;
  };

public:
  TaskIndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            std::shared_ptr<Operator> op,
            std::function<std::list<Match> (nodeid_t)> matchGeneratorFunc,
            unsigned maxNumfOfTasks = std::thread::hardware_concurrency(),
            std::shared_ptr<ThreadPool> threadPool = std::make_shared<ThreadPool>(std::thread::hardware_concurrency()));

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~TaskIndexJoin();
private:

  std::shared_ptr<Iterator> lhs;
  const size_t lhsIdx;
  const unsigned maxNumfOfTasks;

  std::shared_ptr<ThreadPool> workerPool;

  std::list<std::future<std::list<MatchPair>>> taskBuffer;
  size_t taskBufferSize;
  std::list<MatchPair> matchBuffer;

  std::function<std::list<MatchPair>(const std::vector<Match>& )> taskBufferGenerator;

private:
  bool fillTaskBuffer();
  bool nextMatchBuffer();
};
}

