/* 
 * File:   IdenticalCoverage.h
 * Author: thomas
 *
 * Created on 8. Januar 2016, 13:58
 */

#ifndef IDENTICALCOVERAGE_H
#define IDENTICALCOVERAGE_H

#include "../operator.h"

namespace annis
{

class IdenticalCoverage : public Operator
{
public:
  IdenticalCoverage();
  IdenticalCoverage(const IdenticalCoverage& orig) = delete;
  
  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);
  virtual bool isReflexive() {return false;};
  
  virtual ~IdenticalCoverage();
private:

};

} // end namespace annis

#endif /* IDENTICALCOVERAGE_H */

