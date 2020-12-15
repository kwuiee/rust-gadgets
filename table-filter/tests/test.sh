# include in `more` mode
i=`cargo run -- --include 'Chromosome/scaffold name=1' table.tsv | wc -l | cut -d ' ' -f 1`
[[ $i == 100 ]] && echo "more & include pass" || echo "more & include failed: 100/$i"

# include in `less` mode
i=`cargo run -- --less --include 'Chromosome/scaffold name=^1$' table.tsv | wc -l | cut -d ' ' -f 1`
[[ $i == 8 ]] && echo "less & include pass" || echo "less & include failed: 8/$i"

# exclude in `more` mode
# select autosomes, sample value >= 10
i=`cargo run -- --exclude 'Chromosome/scaffold name=(?i)^(?:chr|)[^\d]+$;sample=^\d\.\d+$' table.tsv | wc -l | cut -d ' ' -f 1`
[[ $i == 31 ]] && echo "more & exclude pass" || echo "more & exclude failed: 31/$i"

# exclude in `less` mode
# select autosomes, sample value >= 10
i=`cargo run -- --less --exclude 'Chromosome/scaffold name=(?i)^(?:chr|)[^\d]+$;sample=^\d\.\d+$' table.tsv | wc -l | cut -d ' ' -f 1`
[[ $i == 1 ]] && echo "less & exclude pass" || echo "less & exclude failed: 1/$i"

# less & include & exclude
i=`cargo run -- --less --include 'Associated Gene Name=^TMEM' --exclude 'sample=^\d\.\d+$' table.tsv | wc -l | cut -d ' ' -f 1`
[[ $i == 2 ]] && echo "less & include & exclude pass" || echo "less & include & exclude failed: 2/$i"

# less & noheader & include
i=`cargo run -- --less --no-header --include '1=\d\d\.\d+' noheader.tsv | wc -l`
[[ $i == 1 ]] && echo "less & noheader & include pass" || echo "less & noheader & include failed: 1/$i"
