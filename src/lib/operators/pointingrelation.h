#ifndef POINTINGRELATION_H
#define POINTINGRELATION_H

#include "../db.h"
#include "../edgedb.h"
#include "../operator.h"
#include <vector>

namespace annis
{

class PointingRelation : public Operator
{
public:
  PointingRelation(const DB& db, std::string ns, std::string name,
                   unsigned int minDistance = 1, unsigned int maxDistance = 1);

  PointingRelation(const DB& db, std::string ns, std::string name,
                   const Annotation& edgeAnno = Init::initAnnotation());

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);

  virtual ~PointingRelation();
private:
  const DB& db;
  std::string ns;
  std::string name;
  unsigned int minDistance;
  unsigned int maxDistance;
  Annotation anyAnno;
  const Annotation edgeAnno;


  std::vector<const EdgeDB*> edb;

  void initEdgeDB();
  bool checkEdgeAnnotation(const EdgeDB *e, nodeid_t source, nodeid_t target);
};
} // end namespace annis
#endif // POINTINGRELATION_H
