#include <annis/edgeannotationstorage.h>

#include <annis/serializers.h>
#include <annis/util/size_estimator.h>

#include <fstream>

#include <boost/archive/binary_iarchive.hpp>
#include <boost/archive/binary_oarchive.hpp>


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

