evaluate_benchmark <- function(fn) {
  
  d = read.csv(fn)
  d <- subset(d, Experiment=='Optimized')
  
  worse <- subset(d, Baseline >= 1.0)
  better <- subset(d, Baseline < 1.0)
  
  h <- sort(d$Baseline)
  #barplot(h, main=paste("speedup distribution for ", fn), log="y")
  q <- quantile(d$Baseline)
  
  return (list(worse=nrow(worse), better=nrow(better), quantile=q, dist=h))
  
}







