#ifndef PRECEDENCE_H
#define PRECEDENCE_H

#include "db.h"
#include "defaultjoins.h"
#include "../helper.h"

#include <list>
#include <stack>

namespace annis
{


class Precedence : public BinaryIt
{
public:
  Precedence(DB &db, std::shared_ptr<AnnoIt> left, std::shared_ptr<AnnoIt> right,
             unsigned int minDistance=1, unsigned int maxDistance=1);
  virtual ~Precedence();

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();

  virtual bool filter(const Match& lhs, const Match& rhs);

private:
  const DB& db;

  TokenHelper tokHelper;
  std::shared_ptr<AnnoIt> left;
  std::shared_ptr<AnnoIt> right;
  unsigned int minDistance;
  unsigned int maxDistance;

  std::shared_ptr<RightMostTokenForNodeIterator> tokIteratorForLeftNode;
  const Annotation& annoForRightNode;

  BinaryIt* actualJoin;
  std::stack<Match, std::list<Match>> currentMatches;
  BinaryMatch currentMatchedToken;

  const EdgeDB* edbLeft;
  const EdgeDB* edbOrder;
  bool tokenShortcut;

};





} // end namespace annis

#endif // PRECEDENCE_H
