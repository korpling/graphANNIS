#pragma once

#include <annis/iterators.h>
#include <annis/annosearch/annotationsearch.h>
#include <memory>
#include <functional>

namespace annis {
class UnaryFilter : public EstimatedSearch
{
public:
  UnaryFilter(std::shared_ptr<EstimatedSearch> delegate, std::function<bool(const Match&)> filterFunc);

  virtual bool next(Match& m) override;
  virtual void reset() override;

  virtual std::int64_t guessMaxCount() const override { delegate->guessMaxCount(); }

  virtual std::string debugString() const override {return delegate->debugString();}

private:

  std::shared_ptr<EstimatedSearch> delegate;
  std::function<bool(const Match &)> filterFunc;

};

}

