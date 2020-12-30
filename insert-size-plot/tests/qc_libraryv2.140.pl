=head1 Destcription

		use for coverage stat after alignment

=head1 Usage
		
		l:s	<file list:bam_sort  bam_rmdup sampletype>
		r:s	<target region:bed format file or exome/medicine/germline>	
		q:s	<outdir>
		h|help	show shis message

=head1 version

		Name			: coverage_stat.pl
		Author			: Genetron Health(wangxy, maxy, ligy, wangc)
		Version			: v1.0
		Created On		: 21/11 2014
		Last Modified On	: 01/04 2015

=cut

#!/usr/bin/perl
use warnings;
use File::Basename;
use Getopt::Long;

my ($list, $region, $out, $bed_design, $size_design, $help);
GetOptions(
		"l:s"		=>\ $list,
		"r:s"		=>\ $region,
		"q:s"		=>\ $qcfile,
		"h|help"	=>\ $help
);
#$outdir =~ s/.tar.gz$//;
die `pod2text $0` unless(defined $list && defined $region && defined $qcfile);
my $outdir = './qc';
`mkdir $outdir`;
############################# database declaration #############################
$bed_design = $region;
open REG,"$region" or die $!;
while(<REG>){
	chomp;
	next if(/^$/);
	my @info=split (/\t/,$_);
	$size_design += $info[2] - $info[1] + 1;
	}


############################# main program ##############################
my %Report;
my %insert;
my $sampleID;

if(defined $list){
	open L, $list or die $!;
		while(<L>){
			chomp;
			die `check list format` unless($_ =~ /\S+\s+\S+/);
			my $bam_raw = (split /\s+/, $_)[0];
			my $bam_rmdup = (split /\s+/, $_)[1];
			my $bam_key = (split /\s+/, $_)[2];
			&QC_STATISTIC($bam_raw, $bam_rmdup, $bam_key);
			&INSERT_STAT($bam_rmdup,$bam_key);
			&SEQUENCING_QUALITY($bam_rmdup);
			$bam_key =~ s/N$|T$//g;
			$sampleID = $bam_key;
	}
}
else{
	die `input file or list of files`;
}
&OUTPUT;

############################# sub model ##############################
########QC_stat
sub QC_STATISTIC{
	###initialization
	my(@general, $reads_sum, $reads_rmdup_sum, $reads_dup_sum, $dup_rate, $sample_type);
	my($reads_mapgenome_sum, $base_mapgenome_sum, $base_rmdup_mapgenome_sum, $reads_rmdup_mapgenome_sum, $base_maptarget_sum, $base_maptarget_rmdup_sum, $dup_rate_tr, $coverage, $coverage_p5, $coverage_p10,$coverage_p50,$coverage_p100,$coverage_p140,$coverage_p200,$coverage_p700,$coverage_hot_p700);
	my($reads_mapgenome_fraction, $coverage_fraction,$coverage_p5_fraction, $coverage_p10_fraction,$coverage_p50_fraction,$coverage_p100_fraction,$coverage_p140_fraction,$coverage_p200_fraction,$coverage_p700_fraction,$coverage_hot_p700_fraction);
	my($base_maptarget_fraction, $base_maptarget_rmdup_fraction, $high_quality_depth, $high_quality_distinct_depth);

	###input file
	my $file_input = $_[0];
	my $file_rmdup_input = $_[1];
	my $file_keyname = $_[2];
	###sample type
	if($region eq "germline"){
		$sample_type = "Normal DNA";
	}else{
		if($file_keyname =~ /N$/){
			$sample_type = "Normal DNA";
		}elsif($file_keyname =~ /T$/){
			$sample_type = "Tumor DNA";
		}
	}

	###general statistics
	@general1 = `samtools flagstat $file_input`;
	@general2 = `samtools flagstat $file_rmdup_input`;
	$reads_sum = (split /\s+/, $general1[0])[0];
	$reads_rmdup_sum = (split /\s+/, $general2[0])[0];
	$reads_dup_sum = $reads_sum - $reads_rmdup_sum;
	$dup_rate = sprintf "%0.2f", ($reads_dup_sum/$reads_sum*100);
	$dup_rate = $dup_rate."\%";

	###calculate coverage bases
	##deal whith bam_raw
	my $basefile = basename($file_input);
	my $bed_file = $basefile;
	my $depth_file = $basefile;
	$bed_file =~ s/bam$/bed/g;
	$depth_file =~ s/bam$/depth/g;
	open BAM,"bedtools bamtobed -i $file_input |" || die $!;
	open BED,">$outdir/$bed_file " || die $!;
	while(<BAM>){
		chomp;
		my $bed_line = $_;
		my @line1 = split /\s+/,$bed_line;
		#$base_mapgenome_sum += $line1[2]-$line1[1]+1; #####151
		$base_mapgenome_sum += $line1[2]-$line1[1]; #####150
		#print "$base_mapgenome_sum\n";######
		$reads_mapgenome_sum ++;
		print BED "$bed_line\n";
	}
	close BAM;
	close BED;

	open CRG,"bedtools coverage -a $outdir/$bed_file -b $bed_design  -d |" || die $!;
	open DEP,">$outdir/$depth_file " || die $!;
	while(<CRG>){
		chomp;
		my $depth_line = $_;
		my @cut1 = split /\s+/,$_;
		$base_maptarget_sum += $cut1[4];
		print DEP "$depth_line\n";
	}
	close CRG;
	close DEP;

	system ("rm $outdir/$bed_file");

	$base_maptarget_fraction = $base_maptarget_sum/$base_mapgenome_sum*100;
	$base_maptarget_fraction = sprintf ("%0.2f", $base_maptarget_fraction)."\%";
	$reads_mapgenome_fraction = $reads_mapgenome_sum/$reads_sum*100;
	$reads_mapgenome_fraction = sprintf ("%0.2f", $reads_mapgenome_fraction)."\%";

	##deal whith bam_rmdup
	open AIDHOT, "/bioapp/aidkit.sort1.uniq.list" || die $!;
	my %aidhash;
	while (<AIDHOT>) {
		chomp;
		my @l=split/\t/,$_;
		$aidhash{$l[0]}{$l[1]}=1;
	}
	close AIDHOT;
	my $basefile_rmdup = basename($file_rmdup_input);
	my $bed_rmdup_file = $basefile_rmdup;
	my $depth_rmdup_file = $basefile_rmdup;
	$bed_rmdup_file =~ s/bam$/bed/g;
	$depth_rmdup_file =~ s/bam$/depth/g;
    open RBAM,"bedtools bamtobed -i $file_rmdup_input |" || die $!;
	open RBED,">$outdir/$bed_rmdup_file " || die $!;
	while(<RBAM>){
		chomp;
		my $bed_line = $_;
		my @line1 = split /\s+/,$bed_line;
		#$base_mapgenome_sum += $line1[2]-$line1[1]+1; #####151
		$base_rmdup_mapgenome_sum += $line1[2]-$line1[1]; #####150
		#print "$base_mapgenome_sum\n";######
		$reads_rmdup_mapgenome_sum ++;
		print RBED "$bed_line\n";
	}
	close RBAM;
	close RBED;

	#`bedtools bamtobed -i $file_rmdup_input > $bed_rmdup_file`;
	open RCRG,"bedtools coverage -a $outdir/$bed_rmdup_file -b $bed_design -d |" || die $!;
	open RDEP,">$outdir/$depth_rmdup_file " || die $!;
    
	my %depth;
	while(<RCRG>){
		chomp;
		my $depth_rmdup_line = $_;
		print RDEP "$depth_rmdup_line\n";

		my @cut2 = split /\s+/,$_;
		$base_maptarget_rmdup_sum += $cut2[4];
		$depth{$cut2[4]}++;
        my $pos=$cut2[1]+$cut2[3]-1;
		#calculate bases coverage and bases >5,10,50,100,200 reads
		$coverage++ if($cut2[4]>0);
		$coverage_p5++ if($cut2[4]>4);
		$coverage_p10++ if($cut2[4]>9);
		$coverage_p50++ if($cut2[4]>49);
		$coverage_p100++ if($cut2[4]>99);
		$coverage_p140++ if($cut2[4]>139);
		$coverage_p200++ if($cut2[4]>199);
		$coverage_p700++ if($cut2[4]>699);
		if ($aidhash{$cut2[0]}{$pos} && $cut2[4]>699) {
			$coverage_hot_p700++;

			# body...
		}
	}
	close RCRG;
	close RDEP;

	system ("rm $outdir/$bed_rmdup_file");

	$base_maptarget_rmdup_fraction = $base_maptarget_rmdup_sum/$base_rmdup_mapgenome_sum*100;
	$base_maptarget_rmdup_fraction = sprintf ("%0.2f", $base_maptarget_rmdup_fraction)."\%";
	$dup_rate_tr = ($base_maptarget_sum - $base_maptarget_rmdup_sum)/$base_maptarget_sum*100;
	$dup_rate_tr = sprintf ("%0.2f", $dup_rate_tr)."\%";
	$coverage_fraction = $coverage/$size_design*100;
	$coverage_fraction = sprintf ("%0.2f", $coverage_fraction)."\%";
	$coverage_p5_fraction = $coverage_p5/$size_design*100;
	$coverage_p5_fraction = sprintf ("%0.2f", $coverage_p5_fraction)."\%";
	$coverage_p10_fraction = $coverage_p10/$size_design*100;
	$coverage_p10_fraction = sprintf ("%0.2f", $coverage_p10_fraction)."\%";
	$coverage_p50_fraction = $coverage_p50/$size_design*100;
	$coverage_p50_fraction = sprintf ("%0.2f", $coverage_p50_fraction)."\%";
	$coverage_p100_fraction = $coverage_p100/$size_design*100;
	$coverage_p100_fraction = sprintf ("%0.2f", $coverage_p100_fraction)."\%";
	$coverage_p140_fraction = $coverage_p140/$size_design*100;
	$coverage_p140_fraction = sprintf ("%0.2f", $coverage_p140_fraction)."\%";
	$coverage_p200_fraction = $coverage_p200/$size_design*100;
	$coverage_p200_fraction = sprintf ("%0.2f", $coverage_p200_fraction)."\%";
	$coverage_p700_fraction = $coverage_p700/$size_design*100;
	$coverage_p700_fraction = sprintf ("%0.2f", $coverage_p700_fraction)."\%";
	$coverage_hot_p700_fraction = $coverage_hot_p700/110*100;
	$coverage_hot_p700_fraction = sprintf ("%0.2f", $coverage_hot_p700_fraction)."\%";


	$high_quality_depth = $base_maptarget_sum/$size_design;
	$high_quality_depth = sprintf ("%0.2f", $high_quality_depth);
	$high_quality_distinct_depth = $base_maptarget_rmdup_sum/$size_design;
	$high_quality_distinct_depth = sprintf ("%0.2f", $high_quality_distinct_depth);

	###report result
	$Report{"file_keyname"} .= $file_keyname."\t";
	$Report{"sample_type"} .= $sample_type."\t";
	$Report{"reads_sum"} .= $reads_sum."\t";
	$Report{"reads_dup_sum"} .= $reads_dup_sum."\($dup_rate\)"."\t";
	$Report{"duprate_target"} .= $dup_rate_tr."\t";
	$Report{"base_mapgenome_sum"} .= $base_mapgenome_sum."\t";
	$Report{"reads_mapgenome_sum"} .= $reads_mapgenome_sum."\t";
	$Report{"reads_mapgenome_fraction"} .= $reads_mapgenome_fraction."\t";
	$Report{"base_maptarget_sum"} .= $base_maptarget_sum."\t";
	$Report{"base_maptarget_rmdup_sum"} .= $base_maptarget_rmdup_sum."\t";
	$Report{"base_maptarget_fraction"} .= $base_maptarget_fraction."\t";
	$Report{"base_maptarget_rmdup_fraction"} .= $base_maptarget_rmdup_fraction."\t";
	$Report{"coverage_fraction"} .= $coverage_fraction."\t";
	$Report{"coverage_p5"} .= $coverage_p5."\t";
	$Report{"coverage_p5_fraction"} .= $coverage_p5_fraction."\t";
	$Report{"coverage_p10"} .= $coverage_p10."\t";
	$Report{"coverage_p10_fraction"} .= $coverage_p10_fraction."\t";
	$Report{"coverage_p50_fraction"} .= $coverage_p50_fraction."\t";
	$Report{"coverage_p100_fraction"} .= $coverage_p100_fraction."\t";
	$Report{"coverage_p140_fraction"} .= $coverage_p140_fraction."\t";
	$Report{"coverage_p200_fraction"} .= $coverage_p200_fraction."\t";
	$Report{"coverage_p700_fraction"} .= $coverage_p700_fraction."\t";
	$Report{"coverage_hot_p700_fraction"} .= $coverage_hot_p700_fraction."\t";
	$Report{"high_quality_distinct_depth"} .= $high_quality_distinct_depth."\t";
	$Report{"high_quality_depth"} .= $high_quality_depth."\t";

	###compute depth frenquency
	open DF,">$outdir/$file_keyname.depth_frequency.xls" || die $!;
	my $maxCov = 0;
	$depth{"0"} = $size_design - $coverage;
	foreach my $depth (sort {$a<=>$b} keys %depth){
		my $per = $depth{$depth}/$size_design;
		$maxCov = $per if($per > $maxCov);
		print DF "$depth\t$per\t$depth{$depth}\n";
	}
	close(DF);

	&DRAW_DEPTH($maxCov, $high_quality_depth, $file_keyname );
}

sub DRAW_DEPTH{
	my $ylim = 100*$_[0];
	my $cutoff = $_[1];
	my $prefix = $_[2];
	my ($xbin,$ybin);
	$ylim = int($ylim) + 1;
	if($ylim <= 3){
		$ybin = 0.5;
	}else{
		$ybin = 1;
	}
	my $xlim = 0;
	if($cutoff < 30){
		$xlim = 100;
		$xbin = 20;
	}elsif($cutoff < 50){
		$xlim = 160;
		$xbin = 20;
	}elsif($cutoff < 120){
		$xlim = 250;
		$xbin = 50;
	}else{
		$xlim = 600;
		$xbin = 100;
	}
	histPlot("$outdir/$prefix\.histPlot", "$outdir/$prefix\.depth_frequency.xls");
}

sub histPlot {
	my ($outFile, $dataFile) = @_;
	my $figFile = "$outFile.png";
	my $Rline=<<Rline; 
		png(file="$figFile",width = 400, height = 400)
		rt <- read.table("$dataFile")
		opar <- par()
											        
		rt_nrow = nrow(rt)
		mean_cvg = round(sum(rt\$V1*rt\$V3)/sum(rt\$V3),2)
		median_cvg = median(rep(rt\$V1, rt\$V3))
		by = length(unlist(strsplit(split="",as.character(ceiling(mean_cvg)))))
		by = 10^(by-1)
																							        
		x_lim = median_cvg*2
		x_lim = which(rt\$V2[x_lim:rt_nrow] < 10^-4)[1] + x_lim
		x_lim = floor(x_lim / by) * by

		if(x_lim>nrow(rt)){
			x_lim=nrow(rt)
		}
								                
		t=sum(rt\$V2[(x_lim+1):length(rt\$V2)])
		y=rt\$V2[1:x_lim]
		y <- y*100
		y_lim = round(max(y),1) + 0.3  

		y=c(y,t*100)

		if(y_lim <= 1){
			ybin = 0.2;
		}else{
			ybin = 0.5;
		}
													          
		x <- rt\$V1[1:(x_lim+1)]

		par(mar=c(4.5, 4.5, 2.5, 2.5))
		plot(x,y,col="#50AD9F",type='h', lwd=1.5, xlab="测序深度", ylab="比例(%)", bty="l",ylim=c(0,y_lim),xlim=c(0,x_lim), cex.lab=1, cex.axis=1,col.axis=\"#808080\",col.lab=\"#808080\",col.main=\"#808080\",fg=\"#808080\")
		arrows(mean_cvg, 0, mean_cvg , y_lim/2 , col="#FF6A6A", length = 0.1, lwd=2) 
		arrows(median_cvg, 0, median_cvg , y_lim/2 , col="#FFC125", length = 0.1, lwd=2)
		temp = c( paste(sep=" ", 'mean_cvg', '=', mean_cvg) , paste(sep=" ", 'median_cvg', '=', median_cvg))
		legend("topleft", temp , text.col=c('#FF6A6A','#FFC125'), ncol=1, bty='n', cex=1)
												          
		par(opar)
		dev.off()

Rline
		open (ROUT,">$figFile.R");
		print ROUT $Rline;
		close(ROUT);

		system("R -f  $figFile.R");
}

sub INSERT_STAT {
	my $bamfile=$_[0];
	my $prefix=$_[1];
	my %insertsize;
	my $peak_inszvalue=0;
	my $max_freqnum=0;

	open BAM,"samtools view $bamfile -q 30 -F 0x404 | " or die $!;
	open INSD, ">", "$outdir/$prefix.bwa.insertSize.xls" or die $!;
	while(<BAM>){
		chomp;
		my $line=$_;
		next if(/^\s*$/ || /^#/);
		my @_F=split /\t/; 
		$insertsize{abs($_F[8])}++;
		my $ins=abs($_F[8]);
		print INSD "$ins\n";
	}
	close BAM;

	foreach my $insize (sort {$a<=>$b} keys %insertsize){
		print INSD "$insertsize{$insize}\n";
		next if($insize == 0);
		$peak_inszvalue = $insertsize{$insize} > $max_freqnum ? $insize:$peak_inszvalue;
		$max_freqnum = $insertsize{$insize} > $max_freqnum ? $insertsize{$insize}:$max_freqnum;
	}
	$insert{"peak_value"} .= $peak_inszvalue."\t";
	close INSD;

	my $Rline=<<Rline;
	a<-read.table("$outdir/$prefix.bwa.insertSize.xls")
	names(a)<-c("isize")
	f<-a\$isize<500 & a\$isize>0
	png("$outdir/$prefix.bwa.insertSize.png",width=400,height=400)
	plot(density(a\$isize[f]),xlim=c(0,500),col="#50AD9F",type="l",lwd=2,xlab="插入片段大小（bp）",ylab="比例",main="插入片段大小分布图",cex.lab=1,cex.axis=1,cex.main=1,col.axis=\"#808080\",col.lab=\"#808080\",col.main=\"#808080\",fg=\"#808080\")
	dev.off()
Rline
	open (ISOUT,">","$outdir/$prefix.bwa.insertSize.R");
	print  ISOUT $Rline;
	close ISOUT;
	system("R -f $outdir/$prefix.bwa.insertSize.R");
}

sub SEQUENCING_QUALITY {
	my $bamfile = $_[0];
	my ($count_read, $NM, $cigar, $orig_cigar, $total_NM, $match_base, $total_match_base, $base, $total_base, $read_base, $total_read_base, $mismatch_match, $read_match_rate, $aver_read);

	open BAM,"bedtools bamtobed -cigar -i $bamfile -tag NM |" || die $!;
	while (<BAM>){
		chomp;
		$total_match_base = 0;
		$total_base = 0;
		$total_read_base = 0;
		$count_read ++;
		$NM = (split /\t/, $_)[4];
		$cigar = (split /\t/, $_)[6];
		$orig_cigar = $cigar;
		$total_NM += $NM;
		while ($cigar =~ s/(\d+)[MI]//){
			$match_base += $1;
		}
		$total_match_base += $match_base;
		$cigar = $orig_cigar;
		while ($cigar =~ s/(\d+)[MIS]//){
			$base += $1;
		}
		$total_base += $base;
		$cigar = $orig_cigar;
		while ($cigar =~ s/(\d+)[MISH]//){
			$read_base += $1;
		}
		$total_read_base += $read_base;
	}
	close BAM;
	$mismatch_match = sprintf ("%.2f", ($total_NM / $total_match_base * 100))."%";
	$read_match_rate = sprintf ("%.2f", ($total_match_base / $total_base * 100))."%";
	$aver_read = sprintf "%.0f", ($total_read_base / $count_read);

	$Report{"mismatch_match"} .= $mismatch_match."\t";
        $Report{"read_match_rate"} .= $read_match_rate."\t";
        $Report{"aver_read"} .= $aver_read."\t";
}

sub OUTPUT {
	###print panel size
	open REPORT, ">"."$qcfile" or die $!;
	my $report;
	$report .= "Total panel size (bases): $size_design bp\n\n";

	###print general
	$report .= "Samples\t".$Report{"file_keyname"}."\n";
	$report .= "Sample Type\t".$Report{"sample_type"}."\n";
	$report .= "Total Reads\t".$Report{"reads_sum"}."\n";
	$report .= "Total Duplicate Reads\t".$Report{"reads_dup_sum"}."\n";
	$report .= "Duplicate Rate of Target Region\t".$Report{"duprate_target"}."\n";
	$report .= "InsertSize Estimation\t".$insert{"peak_value"}."\n";

	###print capture rate
	$report .= "\nCapture coverage\n";
	$report .= "Reads mapped to genome\t".$Report{"reads_mapgenome_sum"}."\n";
	$report .= "Sequenced bases mapped to genome (bp)\t".$Report{"base_mapgenome_sum"}."\n";
	$report .= "Fraction mapped to genome\t".$Report{"reads_mapgenome_fraction"}."\n";
	$report .= "Sequenced bases mapped to Target Regions (bp)\t".$Report{"base_maptarget_sum"}."\n";
	$report .= "Fraction of Sequenced Bases Mapped to Target Regions\t".$Report{"base_maptarget_fraction"}."\n";
	$report .= "Sequenced bases mapped to Target Regions (bp) (after duplicates removed)\t".$Report{"base_maptarget_rmdup_sum"}."\n";
	$report .= "Fraction of sequenced bases mapped to Target Regions (bp) (after duplicates removed)\t".$Report{"base_maptarget_rmdup_fraction"}."\n";

	###times target 
	$report .= "\nCoverage in bases\n";
	$report .= "Fraction of bases in target regions with at least 1 unique reads\t".$Report{"coverage_fraction"}."\n";
	$report .= "Bases in target regions with at least 5 unique reads (bp)\t".$Report{"coverage_p5"}."\n";
	$report .= "Fraction of bases in target regions with at least 5 unique reads\t".$Report{"coverage_p5_fraction"}."\n";
	$report .= "Bases in target regions with at least unique 10 reads (bp)\t".$Report{"coverage_p10"}."\n";
	$report .= "Fraction of bases in target regions with at least 10 unique reads\t".$Report{"coverage_p10_fraction"}."\n";
	$report .= "Fraction of bases in target regions with at least 50 unique reads\t".$Report{"coverage_p50_fraction"}."\n";
	$report .= "Fraction of bases in target regions with at least 100 unique reads\t".$Report{"coverage_p100_fraction"}."\n";
	$report .= "Fraction of bases in target regions with at least 140 unique reads\t".$Report{"coverage_p140_fraction"}."\n";
	$report .= "Fraction of bases in target regions with at least 200 unique reads\t".$Report{"coverage_p200_fraction"}."\n";
	$report .= "Fraction of bases in target regions with at least 700 unique reads\t".$Report{"coverage_p700_fraction"}."\n";
	$report .= "Fraction of bases in target regions with at least 700 unique reads(AIDKIT Hotspot)\t".$Report{"coverage_hot_p700_fraction"}."\n";

	###depth
	$report .= "\nAverage depth\n";
	$report .= "Average Number of Total High Quality Sequences at Each Base\t".$Report{"high_quality_depth"}."\n";
	$report .= "Average Number of Distinct High Quality Sequences at Each Base\t".$Report{"high_quality_distinct_depth"}."\n"; 
	
	###sequencing quality
	$report .= "\nSequencing quality\n";
	$report .= "Ratio of mismatched bases to matched bases in matched region\t".$Report{"mismatch_match"}."\n";
	$report .= "Rate of matched bases in total reads\t".$Report{"read_match_rate"}."\n";
	$report .= "Average length of total reads (bp)\t".$Report{"aver_read"}."\n";
	
	###close
	print REPORT $report;
	close REPORT;
}
