#include <annis/edgeannotationstorage.h>

#include <annis/util/size_estimator.h>

#include <fstream>

using namespace annis;

EdgeAnnotationStorage::EdgeAnnotationStorage()
{

}

EdgeAnnotationStorage::~EdgeAnnotationStorage()
{

}

void EdgeAnnotationStorage::clear()
{
  edgeAnnotations.clear();
}


size_t EdgeAnnotationStorage::estimateMemorySize()
{
  return
      + size_estimation::element_size(edgeAnnotations)
      + sizeof(EdgeAnnotationStorage);
}

