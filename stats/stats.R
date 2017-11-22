library(ggplot2)
library(dRank)

gm_mean = function(x) {
	exp(mean(log(x)))
}

tech_scores <- seq(0.97, 0.27, -(0.97 - 0.27) / 11)
tech_weights <- tech_scores / sum(tech_scores)
tech_names <- c("0.97", "0.91", "0.84", "0.78", "0.72", "0.65", "0.59", "0.53", "0.46", "0.40", "0.34", "0.27")
tech <- data.frame(sample = tech_names, score = tech_scores, weight = tech_weights)

cw_calculate = function(file) {
	df <- read.csv(file, header = TRUE, colClasses = c("item_name" = "character"))

	samples <- unique(df$item_name)
	users <- unique(df$user)

	mean <- aggregate(x = df[c("weight")], by = list(sample = df$item_name), FUN = gm_mean)
	mean$norm <- mean$weight / sum(mean$weight)
	mean <- mean[with(mean, order(-weight)), ]
	mean$rank <- 1:length(samples)

	list(
		df = df,
		samples = samples,
		users = users,
		mean = mean
	)
}

cw_stats = function(file) {
	cw <- cw_calculate(file)

	cat("Num samples:", length(cw$samples), "\n")
	cat("Num participants:", length(cw$users), "\n")

	ranking_human <- cw$mean[with(cw$mean, rev(order(sample))), ]$weight
	ranking_code <- tech_weights

	print(cor.test(
		ranking_human,
		ranking_code,
		method = "kendall",
		alternative = "greater"
	))

	tmp <- cw$df[with(cw$df, order(item_name, user)), ]
	X <- matrix(
		tmp$weight,
		nrow = length(cw$users),
		ncol = length(cw$samples),
	)
	y <- sort(ranking_code)
	print(dRank(y, X, B = 10000))
}

cw_plot_weights = function(file) {
	cw <- cw_calculate(file)

	print(tech)

  ggplot() +
    # ylim(0, 1) +
    geom_point(data = cw$df, aes(item_name, weight), size = 0.5, alpha = 0.2, color = "black") +
    geom_point(data = cw$mean, aes(sample, weight), size = 2.0, alpha = 1.0, color = "blue") +
    geom_point(data = tech, aes(sample, weight), size = 2.0, alpha = 1.0, color = "red")
}

cw_plot_aggregate = function(file) {
	cw <- cw_calculate(file)
	# Order bars by mean weight
	cw$mean$sample <- factor(cw$mean$sample, levels = cw$mean$sample)

	ggplot(
		data=cw$mean,
		aes(sample, weight)
	) +
		geom_bar(stat="identity")
}
