$RUNNER_NUM = 3
$DOCKER_IMAGE = "dracon_img"
$DOCKER_NETWORK = "dracon_network"
$CONTAINER_PREFIX = "dracon"

# Start docker containers (runners).
for ($i = 1; $i -le $RUNNER_NUM; $i++) {
    Write-Host "Starting runner $CONTAINER_PREFIX$i" -ForegroundColor Black -BackgroundColor Gray
    docker run -dit --name $CONTAINER_PREFIX$i --network $DOCKER_NETWORK --rm -v ${PWD}:/proj -w /proj $DOCKER_IMAGE
}

Write-Host "All $RUNNER_NUM runners started" -ForegroundColor Black -BackgroundColor Gray

docker ps

# Download dependencies and build ping.
for ($i = 1; $i -le $RUNNER_NUM; $i++) {
    Write-Host "Start building `ping` example for runner $CONTAINER_PREFIX$i" -ForegroundColor Black -BackgroundColor Gray
    docker exec -t $CONTAINER_PREFIX$i bash -c 'PATH=$PATH:$HOME/.cargo/bin cargo build --example ping'
}

Write-Host "All $RUNNER_NUM runners have built the example" -ForegroundColor Black -BackgroundColor Gray

# Retrieve and sort containers' IP addresses.
$runner_ip_pairs = docker network inspect $DOCKER_NETWORK | ConvertFrom-Json | ForEach-Object {
    $_.Containers.PSObject.Properties.Value | ForEach-Object {
        [PSCustomObject]@{
            Name      = $_.Name
            IPAddress = ($_.IPv4Address -replace '/', ':')
        }
    } 
} | Sort-Object Name

function Start-DockerExec {
    param(
        [string]$name,
        [string]$ip
    )

    Write-Host "Start running example for runner $name with IP $ip"

    Start-Job -ScriptBlock {
        param($name, $ip)
        docker exec -t $name bash -c "PATH=`$PATH`:`$HOME`/.cargo/bin cargo run --example ping $ip"
    } -ArgumentList $name, $ip
}

$jobs = @()

# Run ping.
for ($i = 1; $i -le $RUNNER_NUM; $i++) {
    $runner_name = "$CONTAINER_PREFIX$i"
    $runner_info = $runner_ip_pairs | Where-Object Name -eq $runner_name
    if ($null -ne $runner_info) {
        $socket_addr = $runner_info.IPAddress
        $job = Start-DockerExec -name $runner_name -ip $socket_addr
        $jobs += $job
    }
    else {
        Write-Host "No matching container found for runner $runner_name" -ForegroundColor Black -BackgroundColor Gray
    }
}

# Wait for all jobs to finish.
Write-Host "Waiting for all jobs to finish..." -ForegroundColor Black -BackgroundColor Gray
$jobs | Wait-Job

# Collect outputs from all jobs.
Write-Host "Collecting outputs from all jobs" -ForegroundColor Black -BackgroundColor Gray
$jobs | ForEach-Object {
    $output = Receive-Job -Job $_

    # Write output to console.
    $output_text = if ($output -is [Array]) { $output -join "`n" } else { $output }
    Write-Host "Output from job $($_.Id): " -ForegroundColor Black -BackgroundColor Gray
    Write-Host $output_text

    # # Save output to file.
    # $id = $_.Id
    # $output_file = "ping_output_$id.log"
    # $output | Out-File -FilePath $output_file
    # Write-Host "Output from job $id saved to $output_file" -ForegroundColor Blue
}

# Clean up jobs.
$jobs | Remove-Job

Write-Host "Stopping all runners" -ForegroundColor Black -BackgroundColor Gray
docker stop $(docker ps -a -q --filter="name=dracon")
