#ifndef PRECEDENCE_H
#define PRECEDENCE_H

#include "db.h"
#include "defaultjoins.h"
#include "../helper.h"
#include "../operator.h"

#include <list>
#include <stack>

namespace annis
{

class Precedence : public Operator
{
public:

  Precedence(const DB& db, unsigned int minDistance=1, unsigned int maxDistance=1);

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);

  virtual ~Precedence();
private:
  TokenHelper tokHelper;
  const EdgeDB* edbOrder;
  const EdgeDB* edbLeft;
  Annotation anyTokAnno;
  Annotation anyNodeAnno;

  unsigned int minDistance;
  unsigned int maxDistance;
};

class LegacyPrecedence : public BinaryIt
{
public:
  LegacyPrecedence(DB &db, std::shared_ptr<AnnoIt> left, std::shared_ptr<AnnoIt> right,
             unsigned int minDistance=1, unsigned int maxDistance=1);
  virtual ~LegacyPrecedence();

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();

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
