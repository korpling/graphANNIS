#ifndef INCLUSION_H
#define INCLUSION_H

#include <set>
#include <list>

#include "../db.h"
#include "../operator.h"
#include "../helper.h"

namespace annis
{

class Inclusion : public Operator
{
public:
  Inclusion(const DB &db);

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);

  virtual bool isReflexive() {return false;}

  virtual ~Inclusion();
private:

  const DB& db;
  const ReadableGraphStorage* gsOrder;
  const ReadableGraphStorage* gsLeftToken;
  const ReadableGraphStorage* gsRightToken;

  Annotation anyNodeAnno;

  TokenHelper tokHelper;


};
} // end namespace annis
#endif // INCLUSION_H
