from load_scenario.sampling import (
    ProcessSample,
    ResourceSample,
    mean_process_metrics,
    parse_remote_sample_line,
)


def test_parse_remote_sample_line() -> None:
    line = "recorder:1.5:40000 mavlink_camera_manager:2.0:50000 zenohd:0.5:12000 system:25.0:0 "
    sample = parse_remote_sample_line(line)
    assert sample.processes["recorder"] == ProcessSample(cpu_percent=1.5, memory_rss_kib=40000)


def test_mean_process_metrics() -> None:
    samples = [
        ResourceSample(
            processes={
                "recorder": ProcessSample(cpu_percent=10.0, memory_rss_kib=100),
                "mavlink_camera_manager": ProcessSample(cpu_percent=20.0, memory_rss_kib=200),
                "zenohd": ProcessSample(cpu_percent=1.0, memory_rss_kib=50),
                "system": ProcessSample(cpu_percent=30.0, memory_rss_kib=0),
            }
        ),
        ResourceSample(
            processes={
                "recorder": ProcessSample(cpu_percent=12.0, memory_rss_kib=110),
                "mavlink_camera_manager": ProcessSample(cpu_percent=18.0, memory_rss_kib=210),
                "zenohd": ProcessSample(cpu_percent=2.0, memory_rss_kib=60),
                "system": ProcessSample(cpu_percent=32.0, memory_rss_kib=0),
            }
        ),
    ]
    means = mean_process_metrics(samples)
    assert means["recorder"].cpu_percent_mean == 11.0
    assert means["recorder"].memory_rss_kib_mean == 105.0
