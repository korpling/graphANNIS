#pragma once

#include <annis/iterators.h>
#include <memory>
#include <functional>

namespace annis {
class UnaryFilter : public AnnoIt
{
public:
  UnaryFilter(std::shared_ptr<AnnoIt> delegate, std::function<bool(const Match&)> filterFunc);

  virtual bool next(Match& m) override;
  virtual void reset() override;

private:

  std::shared_ptr<AnnoIt> delegate;
  std::function<bool(const Match &)> filterFunc;

};

}

