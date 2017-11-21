library(ggplot2)

gm_mean = function(x) {
	exp(mean(log(x)))
}

tech_scores <- seq(0.97, 0.27, -(0.97 - 0.27) / 11)
tech_weights <- tech_scores / sum(tech_scores)

cw_stats = function(file) {
	cw.df <- read.csv(file, header = TRUE, colClasses = c("item_name" = "character"))

	cw.samples <- unique(cw.df$item_name)
	cat("Num samples:", length(cw.samples), "\n")

	cw.users <- unique(cw.df$user)
	cat("Num participants:", length(cw.users), "\n")

	cw.mean <- aggregate(x = cw.df[c("weight")], by = list(sample = cw.df$item_name), FUN = gm_mean)
	cw.mean$norm <- cw.mean$weight / sum(cw.mean$weight)
	cw.mean <- cw.mean[with(cw.mean, order(-weight)), ]
	cw.mean$rank <- 1:length(cw.samples)
	print(cw.mean)

	ranking_human <- cw.mean[with(cw.mean, rev(order(sample))), ]$weight
	ranking_code <- tech_weights
	correlation = cor(ranking_human, ranking_code, method = "kendall")
	cat("Correlation:", correlation, "\n")

	print(cor.test(
		ranking_human,
		ranking_code,
		method = "kendall",
		alternative = "less"
	))

	# Order bars by mean weight
	cw.mean$sample <- factor(cw.mean$sample, levels = cw.mean$sample)

	ggplot(
		data=cw.mean,
		aes(sample, weight)
	) +
		geom_bar(stat="identity")
}
