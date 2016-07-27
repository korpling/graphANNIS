#pragma once

#include <annis/db.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/operators/operator.h>
#include <vector>

namespace annis
{

class AbstractEdgeOperator : public Operator
{
public:
  AbstractEdgeOperator(ComponentType componentType,
      GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name,
      unsigned int minDistance = 1, unsigned int maxDistance = 1);

  AbstractEdgeOperator(
      ComponentType componentType,
      GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name,
      const Annotation& edgeAnno = Init::initAnnotation());

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);

  virtual bool valid() const {return !gs.empty();}
  
  virtual std::string operatorString() = 0;
  
  virtual std::string description() override;
  
  virtual double selectivity() override;


  
  virtual ~AbstractEdgeOperator();
private:
  ComponentType componentType;
  GraphStorageHolder& gsh;
  const StringStorage& strings;
  std::string ns;
  std::string name;
  unsigned int minDistance;
  unsigned int maxDistance;
  Annotation anyAnno;
  const Annotation edgeAnno;

  std::vector<std::shared_ptr<const ReadableGraphStorage>> gs;

  void initGraphStorage();
  bool checkEdgeAnnotation(std::shared_ptr<const ReadableGraphStorage> gs, nodeid_t source, nodeid_t target);
};

} // end namespace annis
